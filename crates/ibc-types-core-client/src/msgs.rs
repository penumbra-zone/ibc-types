//! These are definitions of messages that a relayer submits to a chain. Specific implementations of
//! these messages can be found, for instance, in ICS 07 for Tendermint-specific chains. A chain
//! handles these messages in two layers: first with the general ICS 02 client handler, which
//! subsequently calls into the chain-specific (e.g., ICS 07) client handler. See:
//! <https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics#create>.

mod create_client;
mod misbehaviour;
mod update_client;
mod upgrade_client;

pub use create_client::MsgCreateClient;
pub use misbehaviour::MsgSubmitMisbehaviour;
pub use update_client::MsgUpdateClient;
pub use upgrade_client::MsgUpgradeClient;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ClientMsg {
    CreateClient(MsgCreateClient),
    UpdateClient(MsgUpdateClient),
    Misbehaviour(MsgSubmitMisbehaviour),
    UpgradeClient(MsgUpgradeClient),
}
