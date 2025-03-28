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
