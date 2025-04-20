use crate::{
    ast::NodeId,
    common::Scope,
    hir::*,
};

use super::{Pass, PassContext};

pub struct SimplePathGenPass<'a> {
    ctx: &'a PassContext
}

impl<'a> SimplePathGenPass<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for SimplePathGenPass<'_> {
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

impl Pass for SimplePathGenPass<'_> {
    const NAME: &'static str = "Simple Path Generation";
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
}

impl Visitor for SelfTyReplacePass<'_> {
    fn visit_trait(&mut self, node: &mut Trait, ctx: &mut TraitContext) {
        let mut idens = Vec::new();
        for segment in ctx.scope.segments() {
            let name = self.ctx.names.read().get_id_for_str(&segment.name);

            idens.push(Identifier {
                name: IdenName::Name { name, span: node.span },
                gen_args: None,
                span: node.span,
            })
        }

        let gen_args = node.generics.as_ref().map(|generics| self.get_gen_args_from_params(generics, node.span));

        // TODO: Generic args
        idens.push(Identifier {
            name: IdenName::Name { name: node.name, span: node.span },
            gen_args,
            span: node.span,
        });

        self.self_ty = SelfTyReplaceInfo::Path(Path {
            span: node.span,
            node_id: node.node_id,
            start: PathStart::None,
            idens,
            fn_end: None,
            ctx: PathCtx::new(),
        });

        self.trait_path = None;
        helpers::visit_trait(self, node);
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
}