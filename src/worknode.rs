//! # Worknode
//!
//! This module is for the node in the workflow graph.
//!
//! ## Type of Worknode
//!
//! There are five types of worknode currently (there may be more in the future):
//! 1. Start node: The start point of the workflow graph.
//! 2. End node: The end point of the workflow graph.
//! 3. AI node: The node that call the AI service.
//! 4. local node: The node that run a local script.
//! 5. user node: The node that wait for user input.

pub mod ai_node;

use crate::error::{PilotError, PilotErrorType, PilotResult};

use uuid::Uuid;

#[derive(Debug, Clone)]
/// The enum of the worknode type. This is the core part of the node.
pub enum Worknodecore {
    /// The start node of the workflow graph.
    Start,
    /// The end node of the workflow graph.
    End,
    /// The AI node of the workflow graph.
    AINode(ai_node::AINode),
    /// The local node of the workflow graph.
    Local,
    /// The user node of the workflow graph.
    User,
}

impl Worknodecore {
    /// Excute the worknode.
    pub async fn excute(&mut self, input: String) -> PilotResult<String> {
        match self {
            Self::AINode(node) => node.execute(input).await.map_err(|e| {
                PilotError::new(
                    PilotErrorType::AINodeErr(e),
                    "AI node failed to execute".to_string(),
                )
            }),
            _ => Ok("".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
/// The struct of the worknode.
pub struct Worknode {
    /// The uid of the worknode.
    uid: Uuid,
    /// The core part of the worknode.
    node: Worknodecore,
}

impl Worknode {
    /// Create a new worknode.
    pub fn new(node: Worknodecore) -> Self {
        Self {
            uid: Uuid::new_v4(),
            node,
        }
    }
    /// Excute the worknode.
    pub async fn excute(&mut self, input: String) -> PilotResult<String> {
        self.node.excute(input).await
    }
    /// Get the uid of the worknode.
    pub fn get_uid(&self) -> Uuid {
        self.uid
    }
    /// Get the core part of the worknode.
    pub fn get_node(&self) -> &Worknodecore {
        &self.node
    }
    /// Set the core part of the worknode
    pub fn set_node(&mut self, node: Worknodecore) {
        self.node = node;
    }
}

#[cfg(test)]
mod test {
    use super::ai_node::{
        deepseek::{DeepSeekClient, DeepSeekModel},
        AINode, AIService,
    };
    use super::*;

    use tokio::runtime::Runtime;

    #[test]
    fn ai_worknode_execute_simple() {
        let deepseek_client = DeepSeekClient::new(
            "https://api.deepseek.ai/v1/chat/completions",
            DeepSeekModel::DeepseekChat,
        )
        .api_key_from_file("./api_key.txt")
        .unwrap();

        let ai_node = AINode::new(AIService::DeepSeek {
            client: deepseek_client,
        })
        .role(Some("你是一只可爱的猫娘".to_string()));

        let mut worknode = Worknode::new(Worknodecore::AINode(ai_node));
        let result = worknode.excute("请介绍一下你自己".to_string());
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(result);
        match result {
            Ok(res) => println!("Result: {}", res),
            Err(e) => panic!("Error: {}", e),
        }
    }

    #[test]
    fn ai_worknode_execute() {
        let deepseek_client = DeepSeekClient::new(
            "https://api.deepseek.ai/v1/chat/completions",
            DeepSeekModel::DeepseekChat,
        )
        .api_key_from_file("./api_key.txt")
        .unwrap();

        let ai_node = AINode::new(AIService::DeepSeek {
            client: deepseek_client,
        });

        let mut worknode = Worknode::new(Worknodecore::AINode(ai_node));
        let result = worknode.excute(
            r#"
                {
                    "role": "你是一只可爱的猫娘",
                    "input": "请介绍一下你自己",
                }"#
            .to_string(),
        );
        let rt = Runtime::new().unwrap();
        let result = rt.block_on(result);
        match result {
            Ok(res) => println!("Result: {}", res),
            Err(e) => panic!("Error: {}", e),
        }
    }
}
