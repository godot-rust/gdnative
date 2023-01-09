use quote::{quote, ToTokens};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum RpcMode {
    Disabled,
    Remote,
    RemoteSync,
    Master,
    Puppet,
    MasterSync,
    PuppetSync,
}

impl RpcMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "remote" => Some(RpcMode::Remote),
            "remote_sync" => Some(RpcMode::RemoteSync),
            "master" => Some(RpcMode::Master),
            "puppet" => Some(RpcMode::Puppet),
            "disabled" => Some(RpcMode::Disabled),
            "master_sync" => Some(RpcMode::MasterSync),
            "puppet_sync" => Some(RpcMode::PuppetSync),
            _ => None,
        }
    }
}

impl Default for RpcMode {
    fn default() -> Self {
        RpcMode::Disabled
    }
}

impl ToTokens for RpcMode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            RpcMode::Disabled => tokens.extend(quote!(RpcMode::Disabled)),
            RpcMode::Remote => tokens.extend(quote!(RpcMode::Remote)),
            RpcMode::RemoteSync => tokens.extend(quote!(RpcMode::RemoteSync)),
            RpcMode::Master => tokens.extend(quote!(RpcMode::Master)),
            RpcMode::Puppet => tokens.extend(quote!(RpcMode::Puppet)),
            RpcMode::MasterSync => tokens.extend(quote!(RpcMode::MasterSync)),
            RpcMode::PuppetSync => tokens.extend(quote!(RpcMode::PuppetSync)),
        }
    }
}
