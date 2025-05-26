use std::mem;

use utils::PatternIdenCollection;

use crate::{common::{NameTable, VarInfoHandle, VarScopeId, VariableInfoScopeBuilder}, hir::*};

use super::{Pass, PassContext};

pub struct VariableScopeCollection<'a> {
    ctx:         &'a PassContext,
    var_info:    VariableInfoScopeBuilder,
    scope_stack: Vec<VarScopeId>,
}

impl<'a> VariableScopeCollection<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            var_info: VariableInfoScopeBuilder::new(),
            scope_stack: Vec::new(),
        }
    }
}

impl Visitor for VariableScopeCollection<'_> {
    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        let scope_id = self.var_info.add_scope(node.span, None);
        self.scope_stack.push(scope_id);

        helpers::visit_function(self, node);

        self.scope_stack.pop();
        let builder = mem::take(&mut self.var_info);
        let var_info = builder.finalize(ctx.sym.clone().unwrap());
        let mut var_info_map = self.ctx.var_infos.write();
        let id = var_info_map.add(var_info);
        ctx.var_info = id;
    }

    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        let scope_id = self.var_info.add_scope(node.span, None);
        self.scope_stack.push(scope_id);

        helpers::visit_trait_function(self, node);

        self.scope_stack.pop();
        let builder = mem::take(&mut self.var_info);
        let var_info = builder.finalize(ctx.sym.clone().unwrap());
        let mut var_info_map = self.ctx.var_infos.write();
        let id = var_info_map.add(var_info);
        ctx.var_info = id;
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        let scope_id = self.var_info.add_scope(node.span, None);
        self.scope_stack.push(scope_id);

        helpers::visit_trait_method(self, node);

        self.scope_stack.pop();
        let builder = mem::take(&mut self.var_info);
        let var_info = builder.finalize(ctx.sym.clone().unwrap());
        let mut var_info_map = self.ctx.var_infos.write();
        let id = var_info_map.add(var_info);
        ctx.var_info = id;
    }

    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        let scope_id = self.var_info.add_scope(node.span, None);
        self.scope_stack.push(scope_id);

        helpers::visit_function(self, node);

        self.scope_stack.pop();
        let builder = mem::take(&mut self.var_info);
        let var_info = builder.finalize(ctx.sym.clone().unwrap());
        let mut var_info_map = self.ctx.var_infos.write();
        let id = var_info_map.add(var_info);
        ctx.var_info = id;
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        let scope_id = self.var_info.add_scope(node.span, None);
        self.scope_stack.push(scope_id);

        helpers::visit_method(self, node);

        self.scope_stack.pop();
        let builder = mem::take(&mut self.var_info);
        let var_info = builder.finalize(ctx.sym.clone().unwrap());
        let mut var_info_map = self.ctx.var_infos.write();
        let id = var_info_map.add(var_info);
        ctx.var_info = id;
    }

    //--------------------------------------------------------------

    fn visit_block(&mut self, node: &mut Block) {
        let scope_id = self.var_info.add_scope(node.span, self.scope_stack.last().map(|id| *id));
        node.ctx.var_scope = scope_id;
        self.scope_stack.push(scope_id);

        helpers::visit_block(self, node);

        self.scope_stack.pop();
    }
}

impl Pass for VariableScopeCollection<'_> {
    const NAME: &'static str = "Variable Scope Collection";
}

//==============================================================================================================================

pub struct VariableCollection<'a> {
    ctx:         &'a PassContext,
    info:        Option<VarInfoHandle>,
    scope_stack: Vec<VarScopeId>,
    cur_scope:   VarScopeId,
}

impl<'a> VariableCollection<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            info: None,
            scope_stack: Vec::new(),
            cur_scope: VarScopeId::INVALID,
        }
    }
}

impl VariableCollection<'_> {
    pub fn add_params(&self, info: &VarInfoHandle, params: &mut Vec<FnParam>) {
        let names = self.ctx.names.read();
        let mut info = info.write();
        for param in params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => {
                    let mut iden_collect = PatternIdenCollection::new();
                    iden_collect.visit_pattern(pattern);
                    for iden in iden_collect.is_mut_and_names {
                        let debug_name = names[iden.name].to_string();
                        info.add_var(self.cur_scope, iden.name, debug_name, iden.span, !iden.is_ref & iden.is_mut, false);
                    }
                },
                FnParam::Opt { span, attrs, label, pattern, ty, def } => {
                    let mut iden_collect = PatternIdenCollection::new();
                    iden_collect.visit_pattern(pattern);
                    for iden in iden_collect.is_mut_and_names {
                        let debug_name = names[iden.name].to_string();
                        info.add_var(self.cur_scope, iden.name, debug_name, iden.span, !iden.is_ref & iden.is_mut, false);
                    }
                },
                FnParam::Variadic { span, attrs, name, ty } => {
                    let debug_name = names[*name].to_string();
                    info.add_var(self.cur_scope, *name, debug_name, *span, false, false);
                },
            }
        }
    }
}

impl Visitor for VariableCollection<'_> {
    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        self.cur_scope = VarScopeId::PROCESS_INITIAL;
        self.scope_stack.push(self.cur_scope);

        let var_info_map = self.ctx.var_infos.read();
        let info = var_info_map.get(ctx.var_info);

        self.add_params(&info, &mut node.params);

        self.info = Some(info);
        helpers::visit_function(self, node);

        self.scope_stack.pop();
    }

    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        self.cur_scope = VarScopeId::PROCESS_INITIAL;
        self.scope_stack.push(self.cur_scope);

        let var_info_map = self.ctx.var_infos.read();
        let info = var_info_map.get(ctx.var_info);

        self.add_params(&info, &mut node.params);

        self.info = Some(info);
        helpers::visit_trait_function(self, node);

        self.scope_stack.pop();
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        self.cur_scope = VarScopeId::PROCESS_INITIAL;
        self.scope_stack.push(self.cur_scope);

        let var_info_map = self.ctx.var_infos.read();
        let info = var_info_map.get(ctx.var_info);

        match &node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => {
                let mut info = info.write();
                let mut names = self.ctx.names.write();
                let name = names.add("self");
                info.add_var(self.cur_scope, name, "self".to_string(), *span, !is_ref & is_mut, false);
            },
            FnReceiver::SelfTyped { span, is_mut, ty } => {
                let mut info = info.write();
                let mut names = self.ctx.names.write();
                let name = names.add("self");
                info.add_var(self.cur_scope, name, "self".to_string(), *span, *is_mut, false);
            },
        }

        self.add_params(&info, &mut node.params);

        self.info = Some(info);
        helpers::visit_trait_method(self, node);

        self.scope_stack.pop();
    }

    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        self.cur_scope = VarScopeId::PROCESS_INITIAL;
        self.scope_stack.push(self.cur_scope);

        let var_info_map = self.ctx.var_infos.read();
        let info = var_info_map.get(ctx.var_info);

        self.add_params(&info, &mut node.params);

        self.info = Some(info);
        helpers::visit_function(self, node);

        self.scope_stack.pop();
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        self.cur_scope = VarScopeId::PROCESS_INITIAL;
        self.scope_stack.push(self.cur_scope);

        let var_info_map = self.ctx.var_infos.read();
        let info = var_info_map.get(ctx.var_info);

        match &node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => {
                let mut info = info.write();
                let mut names = self.ctx.names.write();
                let name = names.add("self");
                info.add_var(self.cur_scope, name, "self".to_string(), *span, !is_ref & is_mut, false);
            },
            FnReceiver::SelfTyped { span, is_mut, ty } => {
                let mut info = info.write();
                let mut names = self.ctx.names.write();
                let name = names.add("self");
                info.add_var(self.cur_scope, name, "self".to_string(), *span, *is_mut, false);
            },
        }

        self.add_params(&info, &mut node.params);

        self.info = Some(info);
        helpers::visit_method(self, node);

        self.scope_stack.pop();
    }

    
    //--------------------------------------------------------------

    fn visit_block(&mut self, node: &mut Block) {
        self.scope_stack.push(node.ctx.var_scope);
        self.cur_scope = node.ctx.var_scope;

        helpers::visit_block(self, node);

        self.cur_scope = self.scope_stack.pop().unwrap();
    }

    fn visit_var_decl(&mut self, node: &mut VarDecl) {
        {
            let mut info = self.info.as_ref().unwrap().write();
            let names = self.ctx.names.read();
            let debug_name = names[node.name].to_string();
            info.add_var(self.cur_scope, node.name, debug_name, node.span, node.is_mut, false);
        }
    }
}

impl Pass for VariableCollection<'_> {
    const NAME: &'static str = "Variable Collection";
}