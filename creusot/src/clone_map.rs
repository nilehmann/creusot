use std::ops::ControlFlow;

use indexmap::IndexMap;

use petgraph::graphmap::DiGraphMap;
use petgraph::EdgeDirection::Incoming;
use why3::declaration::{CloneKind, CloneSubst, Decl, DeclClone, Use};
use why3::{Ident, QName};

use rustc_hir::def_id::DefId;
use rustc_middle::ty::{
    self,
    fold::{TypeFoldable, TypeVisitor},
    subst::{InternalSubsts, Subst, SubstsRef},
    AssocKind, ProjectionTy, Ty, TyCtxt, TyKind,
};

use heck::CamelCase;

use crate::ctx::{self, *};
use crate::translation::interface;
use crate::translation::ty::translate_ty;
use crate::util::{self, method_name};

// Prelude modules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreludeModule {
    Int,
    Int32,
    Int64,
    UInt32,
    UInt64,
    Char,
    Single,
    Double,
    Prelude,
    Ref,
    Type,
}

impl PreludeModule {
    fn qname(&self) -> QName {
        match self {
            PreludeModule::Int => QName::from_string("mach.int.Int").unwrap(),
            PreludeModule::Int32 => QName::from_string("mach.int.Int32").unwrap(),
            PreludeModule::Int64 => QName::from_string("mach.int.Int64").unwrap(),
            PreludeModule::UInt32 => QName::from_string("mach.int.UInt32").unwrap(),
            PreludeModule::UInt64 => QName::from_string("mach.int.UInt64").unwrap(),
            PreludeModule::Char => QName::from_string("string.Char").unwrap(),
            PreludeModule::Single => QName::from_string("floating_point.Single").unwrap(),
            PreludeModule::Double => QName::from_string("floating_point.Double").unwrap(),
            PreludeModule::Prelude => QName::from_string("prelude.Prelude").unwrap(),
            PreludeModule::Ref => QName::from_string("Ref").unwrap(),
            PreludeModule::Type => QName::from_string("Type").unwrap(),
        }
    }
}

type CloneNode<'tcx> = (DefId, SubstsRef<'tcx>);

#[derive(Clone)]
pub struct CloneMap<'tcx> {
    tcx: TyCtxt<'tcx>,
    prelude: IndexMap<PreludeModule, bool>,
    names: IndexMap<CloneNode<'tcx>, CloneInfo<'tcx>>,
    count: usize,
    item_type: ItemType,

    // Graph which is used to calculate the full clone set
    clone_graph: DiGraphMap<CloneNode<'tcx>, Option<SubstsRef<'tcx>>>,
    // Index of the last cloned entry
    last_cloned: usize,
}

#[derive(Clone)]
pub struct CloneInfo<'tcx> {
    name: Ident,
    hidden: bool,
    projections: Vec<(DefId, Ty<'tcx>)>,
    cloned: bool,
}

impl CloneInfo<'tcx> {
    fn from_name(name: String) -> Self {
        CloneInfo { name: name.into(), hidden: false, projections: Vec::new(), cloned: false }
    }

    fn hidden(name: Ident) -> Self {
        CloneInfo { name, hidden: true, projections: Vec::new(), cloned: false }
    }

    pub fn add_projection(&mut self, proj: (DefId, Ty<'tcx>)) {
        self.projections.push(proj);
    }

    // TODO: When traits stop holding all functions we can remove the last two arguments
    pub fn qname(&self, tcx: TyCtxt, def_id: DefId) -> QName {
        self.qname_raw(method_name(tcx, def_id))
    }

    fn qname_raw(&self, method: Ident) -> QName {
        QName { module: vec![self.name.clone()], name: method }
    }
}

impl<'tcx> CloneMap<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, item_type: ItemType) -> Self {
        let names = IndexMap::new();
        CloneMap {
            tcx,
            names,
            count: 0,
            prelude: IndexMap::new(),
            item_type,
            clone_graph: DiGraphMap::new(),
            last_cloned: 0,
        }
    }

    pub fn insert(&mut self, mut def_id: DefId, subst: SubstsRef<'tcx>) -> &mut CloneInfo<'tcx> {
        if let Some(it) = self.tcx.opt_associated_item(def_id) {
            if let ty::TraitContainer(_) = it.container {
                def_id = it.container.id()
            }
        };

        let tcx = self.tcx;
        let count = &mut self.count;
        self.names.entry((def_id, subst)).or_insert_with(|| {
            let name_base = tcx.item_name(def_id).as_str().to_camel_case();
            let info = CloneInfo::from_name(format!("{}{}", name_base, count));
            *count += 1;
            info
        })
    }

    pub fn clone_self(&mut self, self_id: DefId) {
        let mut modl = ctx::translate_value_id(self.tcx, self_id);
        let clone_name = if modl.module.is_empty() { modl.name } else { modl.module.remove(0) };
        let subst = InternalSubsts::identity_for_item(self.tcx, self_id);
        self.names.insert((self_id, subst), CloneInfo::hidden(clone_name));
    }

    pub fn import_prelude_module(&mut self, module: PreludeModule) {
        self.prelude.entry(module).or_insert(false);
    }

    pub fn keys(&self) -> impl Iterator<Item = &CloneNode<'tcx>> {
        self.names.keys()
    }

    // Update the clone graph with new entries
    fn update_graph(&mut self, ctx: &mut ctx::TranslationCtx<'_, 'tcx>) {
        // Construct a maximal sharing graph for all dependencies.
        // We build edges between each (function, subst) pair, following the call graph
        // Additionally, when the substitution refers to an associated type, we construct
        // a relevant edge.
        //
        // Along the edge, we include the 'original' substitution, which we can use
        // to build the correct substitution.
        //
        let mut i = self.last_cloned;
        let empty = Self::new(self.tcx, self.item_type);

        while i < self.names.len() {
            let (&key, clone_info) = self.names.get_index(i).unwrap();
            i += 1;

            if clone_info.hidden {
                continue;
            }

            self.clone_graph.add_node(key);
            {
                // Gather all the associated types used in a substitution so we can force an edge
                let mut visitor = ProjectionTyVisitor {
                    f: Box::new(|pty: ProjectionTy<'tcx>| {
                        let trait_id = pty.trait_def_id(self.tcx);
                        self.clone_graph.add_edge((trait_id, pty.substs), key, None);
                    }),
                };
                key.1.visit_with(&mut visitor);

                // If we have an additional projections, check if their type contains a further associated type.
                for (_, t) in &clone_info.projections {
                    t.visit_with(&mut visitor);
                }
            }

            // We don't need to construct the sharing graph if the base node is a logic function.
            if self.item_type.clone_interfaces() {
                continue;
            }

            for dep in ctx.dependencies(key.0).unwrap_or(&empty).keys() {
                let orig = dep.1;
                let dep = (dep.0, dep.1.subst(self.tcx, key.1));
                self.insert(dep.0, dep.1);

                // Skip reflexive edges
                if dep == key {
                    continue;
                }

                debug!("edge {:?} -> {:?}", dep, key);
                self.clone_graph.add_edge(dep, key, Some(orig));
            }
        }
    }

    pub fn to_clones(&mut self, ctx: &mut ctx::TranslationCtx<'_, 'tcx>) -> Vec<Decl> {
        let mut decls = Vec::new();

        use petgraph::visit::{Topo, Walker};
        let empty = Self::new(self.tcx, self.item_type);

        // Update the clone graph with any new entries.
        self.update_graph(ctx);

        self.last_cloned = self.count;

        debug!(
            "dep_graph nodes={} edges={}",
            self.clone_graph.node_count(),
            self.clone_graph.edge_count()
        );

        // Traverse the dependency graph in topological order to create the minimal amount of
        // clones that are needed. This allows us to share all the nodes that are higher up in
        // the dependency graph.
        // TODO: Ensure that if there is a cycle we emit a nice error.

        let mut topo = Topo::new(&self.clone_graph);
        while let Some(node @ (def_id, subst)) = topo.walk_next(&self.clone_graph) {
            debug!("processing node={:?}", node);
            // Though we pass in a &mut ref, it shouldn't actually be possible to add any new entries..
            let mut clone_subst = base_subst(ctx, self, def_id, subst);

            if self.names[&node].cloned {
                continue;
            }
            self.names[&node].cloned = true;

            if self.names[&node].hidden {
                continue;
            }

            // Add all associated type projections to the substitution.
            // We must be ordered topologically *after* whatever occurs on the RHS of the projection
            for proj in &std::mem::take(&mut self.names[&node].projections) {
                let ty = translate_ty(ctx, self, rustc_span::DUMMY_SP, proj.1);
                clone_subst.push(CloneSubst::Type(
                    crate::translation::ty::ty_name(ctx.tcx, proj.0).into(),
                    ty,
                ));
            }

            let node_clones = ctx.dependencies(def_id).unwrap_or_else(|| &empty);
            for (dep, t, &orig_subst) in self.clone_graph.edges_directed(node, Incoming) {
                debug!("s={:?} t={:?} e={:?}", dep, t, orig_subst);
                let prov_info = match orig_subst {
                    Some(subst) => &node_clones.names[&(dep.0, subst)],
                    None => continue,
                };
                // Grab the symbols from all dependencies
                let user_info = &self.names[&dep];
                for sym in exported_symbols(ctx.tcx, dep.0) {
                    let elem = match sym {
                        SymbolKind::Val(n) => {
                            CloneSubst::Val(prov_info.qname_raw(n.clone()), user_info.qname_raw(n))
                        }
                        SymbolKind::Type(t) => CloneSubst::Type(
                            prov_info.qname_raw(t.clone()),
                            why3::mlcfg::Type::TConstructor(user_info.qname_raw(t)),
                        ),
                        SymbolKind::Function(f) => CloneSubst::Function(
                            prov_info.qname_raw(f.clone()),
                            user_info.qname_raw(f),
                        ),
                        SymbolKind::Predicate(p) => CloneSubst::Predicate(
                            prov_info.qname_raw(p.clone()),
                            user_info.qname_raw(p),
                        ),
                    };
                    // If we are in an interface, then we should not attempt to share
                    // dependencies at all.
                    if self.item_type != ItemType::Interface {
                        clone_subst.push(elem);
                    }
                }
            }

            decls.push(Decl::Clone(DeclClone {
                name: cloneable_name(ctx.tcx, def_id, self.item_type.clone_interfaces()),
                subst: clone_subst,
                kind: CloneKind::Named(self.names[&node].name.clone()),
            }));
        }

        self.prelude
            .iter_mut()
            .filter(|(_, v)| **v == false)
            .map(|(p, v)| {
                *v = true;
                p
            })
            .map(|q| Decl::UseDecl(Use { name: q.qname() }))
            .chain(decls.into_iter())
            .collect()
    }
}

// Create the substitution used to clone `def_id` with the rustc substitution `subst`.
pub fn base_subst<'tcx>(
    ctx: &mut TranslationCtx<'_, 'tcx>,
    names: &mut CloneMap<'tcx>,
    def_id: DefId,
    subst: SubstsRef<'tcx>,
) -> Vec<CloneSubst> {
    use heck::SnakeCase;
    use rustc_middle::ty::GenericParamDefKind;

    let trait_params = ctx.tcx.generics_of(def_id);
    let mut clone_subst = Vec::new();

    if subst.is_empty() {
        return Vec::new();
    }

    for ix in 0..trait_params.count() {
        let p = trait_params.param_at(ix, ctx.tcx);
        let ty = subst[ix];
        if let GenericParamDefKind::Type { .. } = p.kind {
            let ty = super::ty::translate_ty(ctx, names, rustc_span::DUMMY_SP, ty.expect_ty());
            clone_subst.push(CloneSubst::Type(p.name.to_string().to_snake_case().into(), ty));
        }
    }

    clone_subst
}

fn cloneable_name(tcx: TyCtxt, def_id: DefId, interface: bool) -> QName {
    let qname = translate_value_id(tcx, def_id);
    use util::ItemType::*;

    // TODO: Refactor.
    match util::item_type(tcx, def_id) {
        Logic | Predicate => {
            if interface {
                // TODO: this should directly be a function...
                QName { module: Vec::new(), name: interface::interface_name(tcx, def_id) }
            } else {
                qname.module_name()
            }
        }
        Interface | Program => {
            QName { module: Vec::new(), name: interface::interface_name(tcx, def_id) }
        }
        Trait | Impl => qname,
        Type => unreachable!(),
    }
}

enum SymbolKind {
    Val(Ident),
    Type(Ident),
    Function(Ident),
    Predicate(Ident),
}

// Gather the list of symbols that are exported from a DefId in the eyes of Creusot.
// In short:
// - All kinds of functions: function name
// - Traits & Impls: All functions in the trait/impl + all associated types
fn exported_symbols(
    tcx: TyCtxt<'tcx>,
    def_id: DefId,
) -> Box<dyn Iterator<Item = SymbolKind> + 'tcx> {
    use util::ItemType::*;
    match util::item_type(tcx, def_id) {
        Logic => Box::new(std::iter::once(SymbolKind::Function(method_name(tcx, def_id)))),
        Predicate => Box::new(std::iter::once(SymbolKind::Predicate(method_name(tcx, def_id)))),
        Interface | Program => Box::new(std::iter::once(SymbolKind::Val(method_name(tcx, def_id)))),
        Trait | Impl => {
            Box::new(tcx.associated_items(def_id).in_definition_order().filter_map(move |a| {
                match a.kind {
                    AssocKind::Fn => match util::item_type(tcx, a.def_id) {
                        Logic => Some(SymbolKind::Function(method_name(tcx, a.def_id))),
                        Predicate => Some(SymbolKind::Predicate(method_name(tcx, a.def_id))),
                        Program => Some(SymbolKind::Val(method_name(tcx, a.def_id))),
                        _ => unreachable!(),
                    },
                    AssocKind::Type => Some(SymbolKind::Type(
                        crate::translation::ty::ty_name(tcx, a.def_id).into(),
                    )),
                    AssocKind::Const => None,
                }
            }))
        }
        Type => unreachable!(),
    }
}

// A basic visitor which can be used to gether ProjectionTys containd in
// a foldable struct
struct ProjectionTyVisitor<'a, 'tcx> {
    f: Box<dyn FnMut(ProjectionTy<'tcx>) + 'a>,
}

impl TypeVisitor<'tcx> for ProjectionTyVisitor<'a, 'tcx> {
    fn tcx_for_anon_const_substs(&self) -> Option<TyCtxt<'tcx>> {
        None
    }

    fn visit_ty(&mut self, t: Ty<'tcx>) -> ControlFlow<Self::BreakTy> {
        match t.kind() {
            TyKind::Projection(t) => (*self.f)(*t),
            _ => {}
        }
        ControlFlow::CONTINUE
    }
}
