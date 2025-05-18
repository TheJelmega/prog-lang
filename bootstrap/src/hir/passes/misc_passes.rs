use crate::{
    ast::NodeId,
    common::{self, LibraryPath, Scope},
    hir::*,
};

use super::{Pass, PassContext};

pub struct SimplePathGen<'a> {
    ctx: &'a PassContext
}

impl<'a> SimplePathGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for SimplePathGen<'_> {
    fn visit_simple_path(&mut self, node: &mut SimplePath) {
        if !node.ctx.path.is_empty() {
            return;
        }

        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for name in &node.names {
            path.push(names[*name].to_string());
        }
        node.ctx.path = path;
    }
}

impl Pass for SimplePathGen<'_> {
    const NAME: &'static str = "Simple Path Generation";
}

//==============================================================================================================================

pub struct EarlyPathGen<'a> {
    ctx: &'a PassContext
}

impl<'a> EarlyPathGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx
        }
    }
}

impl Visitor for EarlyPathGen<'_> {
    fn visit_path(&mut self, node: &mut Path) {
        if !node.ctx.path.is_empty() {
            return;
        }

        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for iden in &node.idens {
            match &iden.name {
                IdenName::Name { name, span } => {
                    path.push(names[*name].to_string());
                },
                IdenName::Disambig { span, trait_path, name, name_span } => todo!(),
            }
        }

        node.ctx.path = path;
    }
}

impl Pass for EarlyPathGen<'_> {
    const NAME: &'static str = "Early Path Generation";
}

enum SelfTyReplaceInfo {
    Invalid,
    Path(Path),
    Type(Box<Type>),
}

pub struct SelfTyReplacePass<'a> {
    ctx: &'a    PassContext,
    self_ty:    SelfTyReplaceInfo,
    trait_path: Option<Box<Path>>,
}

impl<'a> SelfTyReplacePass<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            self_ty: SelfTyReplaceInfo::Invalid,
            trait_path: None,
        }
    }
}

impl SelfTyReplacePass<'_> {
    fn get_gen_args_from_params(&mut self, generics: &GenericParams, span: SpanId) -> Box<GenericArgs> {
        let mut args = Vec::new();
            for gen_param in &generics.params {
                let arg = match gen_param {
                    GenericParam::Type(param) => {
                        let path = Path {
                            span: param.span,
                            node_id: NodeId::INVALID,
                            start: PathStart::None,
                            idens: vec![
                                Identifier {
                                    name: IdenName::Name { name: param.name, span: param.span },
                                    gen_args: None,
                                    span: param.span,
                                }
                            ],
                            fn_end: None,
                            ctx: PathCtx::new(),
                        };

                        let path_ty = Box::new(Type::Path(PathType {
                            span: param.span,
                            node_id: NodeId::INVALID,
                            path,
                            ctx: TypeContext::new(),
                        }));

                        GenericArg::Type(path_ty)
                    },
                    GenericParam::TypeSpec(param) => {
                        GenericArg::Type(param.ty.clone())
                    },
                    GenericParam::Const(param) => {
                        GenericArg::Value(Box::new(Expr::Path(PathExpr::Named {
                            span,
                            node_id: NodeId::INVALID,
                            start: PathStart::None,
                            iden: Identifier {
                                name: IdenName::Name { name: param.name, span: param.span },
                                gen_args: None,
                                span: param.span,
                            },
                        })))
                    },
                    GenericParam::ConstSpec(param) => {
                        GenericArg::Value(Box::new(Expr::Block(BlockExpr {
                            span,
                            node_id: NodeId::INVALID,
                            kind: BlockKind::Normal,
                            block: *param.expr.clone(),
                        })))
                    },
                };
                args.push(arg);
            }

            Box::new(GenericArgs {
                span,
                node_id: NodeId::INVALID,
                args,
            })
    }

    fn create_path_type(&mut self, scope: &Scope, name: NameId, generics: Option<&Box<GenericParams>>, span: SpanId, node_id: NodeId) -> SelfTyReplaceInfo {
        let mut idens = Vec::new();
        for segment in scope.segments() {
            let name = self.ctx.names.read().get_id_for_str(&segment.name);

            idens.push(Identifier {
                name: IdenName::Name { name, span },
                gen_args: None,
                span,
            })
        }

        let gen_args = generics.map(|generics| self.get_gen_args_from_params(generics, span));
        idens.push(Identifier {
            name: IdenName::Name { name, span },
            gen_args,
            span,
        });

        SelfTyReplaceInfo::Path(Path {
            span,
            node_id,
            start: PathStart::None,
            idens,
            fn_end: None,
            ctx: PathCtx::new(),
        })
    }
}

impl Visitor for SelfTyReplacePass<'_> {
    fn visit_struct(&mut self, node: &mut Struct, ctx: &mut StructContext) {
        self.self_ty = self.create_path_type(&ctx.scope, node.name, node.generics.as_ref(), node.span, node.node_id);
        self.trait_path = None;
        helpers::visit_struct(self, node);
    }

    fn visit_tuple_struct(&mut self, node: &mut TupleStruct, ctx: &mut StructContext) {
        self.self_ty = self.create_path_type(&ctx.scope, node.name, node.generics.as_ref(), node.span, node.node_id);
        self.trait_path = None;
        helpers::visit_tuple_struct(self, node);
    }

    fn visit_union(&mut self, node: &mut Union, ctx: &mut UnionContext) {
        self.self_ty = self.create_path_type(&ctx.scope, node.name, node.generics.as_ref(), node.span, node.node_id);
        self.trait_path = None;
        helpers::visit_union(self, node);
    }

    fn visit_adt_enum(&mut self, node: &mut AdtEnum, ctx: &mut AdtEnumContext) {
        self.self_ty = self.create_path_type(&ctx.scope, node.name, node.generics.as_ref(), node.span, node.node_id);
        self.trait_path = None;
        helpers::visit_adt_enum(self, node);
    }

    fn visit_bitfield(&mut self, node: &mut Bitfield, ctx: &mut BitfieldContext) {
        self.self_ty = self.create_path_type(&ctx.scope, node.name, node.generics.as_ref(), node.span, node.node_id);
        self.trait_path = None;
        helpers::visit_bitfield(self, node);
    }
    

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        self.self_ty = if let Type::Path(path_ty) = &*node.ty {
            SelfTyReplaceInfo::Path(path_ty.path.clone())
        } else {
            SelfTyReplaceInfo::Type(node.ty.clone())
        };

        if let Some(trait_path) = &node.impl_trait {
            self.trait_path = Some(Box::new(trait_path.clone()));
        } else {
            self.trait_path = None;
        }

        helpers::visit_impl(self, node);
    }

    //--------------------------------------------------------------

    fn visit_path(&mut self, path: &mut Path) {
        helpers::visit_path(self, path);

        if !matches!(&path.start, PathStart::SelfTy { .. }) {
            return;
        }


        match &self.self_ty {
            SelfTyReplaceInfo::Invalid => {
                path.start = path.start.clone();
            },
            SelfTyReplaceInfo::Path(self_path) => {
                path.start = PathStart::None;

                let mut idens = self_path.idens.clone();
                idens.extend_from_slice(&path.idens);
                path.idens = idens;
            },
            SelfTyReplaceInfo::Type(ty) => {
                path.start = PathStart::Type { ty: ty.clone() };

                if let Some(trait_path) = &self.trait_path {
                    let Some(Identifier{ name: IdenName::Name { name, span }, ..}) = &path.idens.get(0) else { return; };
                    let name = IdenName::Disambig {
                        span: *span,
                        trait_path: trait_path.clone(),
                        name: *name,
                        name_span: *span,
                    };

                    path.idens[0].name = name;
                }
            },
        }



    }
}

impl Pass for SelfTyReplacePass<'_> {
    const NAME: &'static str = "Self Type Replacement";

    fn process(&mut self, hir: &mut Hir) {
        let flags = VisitFlags::Struct
                  | VisitFlags::TupleStruct
                  | VisitFlags::Union
                  | VisitFlags::AdtEnum
                  | VisitFlags::Union
                  | VisitFlags::AnyTrait;

        self.visit(hir, flags);
    }
}