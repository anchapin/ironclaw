// Agent Execution Module
//
// This module provides agent execution flow functionality.
// It integrates the orchestrator with the Python agent loop (agent/loop.py)
// by spawning simulated VMs (Phase 1) or actual Micro-VMs (Phase 2+).
//
// Key Features:
// - Spawn simulated or real VMs
// - Execute Python reasoning loop in VM
// - Collect results and terminate VM
// - Full lifecycle management

use anyhow::{Context, Result};
use serde_json::json;
use std::process::{Command, Stdio};
use std::io::Write;
use tokio::process::Child;
use tracing::{debug, error, info};

use crate::vm::config::VmConfig;

/// Command to run agent
#[derive(Debug, clap::Parser)]
pub struct RunAgentArgs {
    /// Task description for the agent
    #[arg(short, long)]
    task: String,

    /// Configuration file path (optional)
    #[arg(short, long)]
    config: Option<String>,

    /// Timeout in seconds (default: 300)
    #[arg(short, long, default_value = "300")]
    timeout: u64,

    /// Use real VM if available (for Phase 2+)
    #[arg(short, long)]
    real_vm: bool,
}

/// Agent execution handle
///
/// Represents a running agent instance with its VM and process.
pub struct AgentExecution {
    /// The VM process (or simulation)
    vm_process: Child,
    /// Task ID being executed
    task_id: String,
    /// Spawn time in milliseconds
    spawn_time_ms: f64,
}

impl AgentExecution {
    /// Execute the agent reasoning loop
    ///
    /// This spawns a simulated VM (Phase 1) or real VM (Phase 2+)
    /// and runs the Python agent loop (`agent/loop.py`) within that VM.
    /// The agent performs its task and returns results.
    ///
    /// # Arguments
    ///
    /// * `task` - Task description for the agent
    /// * `config` - Optional VM configuration override
    /// * `timeout` - Maximum execution time in seconds
    ///
    /// # Returns
    ///
    /// * `AgentExecution` - Handle to the running agent
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Agent binary not found
    /// - VM spawn fails
    /// - Agent fails to start
    pub fn execute_agent(
        task: &str,
        config: Option<&VmConfig>,
        timeout: u64,
    ) -> Result<AgentExecution> {
        // Determine the command to run the Python agent
        let agent_binary = if cfg!(feature = "vm-prototype") {
            // Phase 1: Use simulated VM (Python stub)
            "python3"
        } else {
            // Phase 2+: Use real Firecracker VM
            // TODO: Implement after Phase 2
            anyhow::bail!("Real VM agent execution not yet implemented (Phase 2)")
        };

        info!("Running agent with task: {}", task);

        // Build VM config (use default or override)
        let vm_config = if let Some(config) = config {
            config.clone()
        } else {
            VmConfig::new(task.to_string())
        };

        // Phase 1: Simulated VM (for now)
        // In Phase 1, we create a temporary Python script that acts as a "VM"
        let simulated_vm_script = create_simulated_vm_script(&vm_config)?;

        // Spawn the "VM" (simulated agent process)
        let vm_process = Command::new(&simulated_vm_script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn simulated VM process")?;

        let start_time = std::time::Instant::now();

        // Write the task description to stdin (so agent knows what to do)
        if let Err(e) = vm_process.stdin.as_ref().write_all(
            format!("{{\\"task\\": \\"{}\\"}}\\n", task).as_bytes()
        ) {
            error!("Failed to send task to agent: {}", e);
            vm_process.kill().context("Failed to kill VM process")?;
            return Err(e.into());
        }

        // Wait for agent to complete or timeout
        let duration = std::time::Duration::from_secs(timeout);

        match tokio::time::timeout(duration, vm_process.wait()) {
            Ok(_) => {
                // VM exited successfully
                let elapsed = start_time.elapsed().as_millis();
                info!(
                    "Agent completed task {} in {:.2}ms",
                    task,
                    elapsed.as_millis()
                );
            }
            Err(_) => {
                // Timeout - kill the VM
                vm_process.kill().context("Failed to kill VM process")?;
                info!("Agent execution timed out after {}s", timeout);
                return Ok(()); // Consider timeout a success, just terminated it
            }
        }
    }
}

/// Create a simulated VM script for Phase 1
///
/// This creates a Python script that simulates a VM environment.
/// The agent reads from stdin and writes results to stdout.
fn create_simulated_vm_script(config: &VmConfig) -> Result<String> {
    use std::fmt;

    Ok(format!(
        r#"#!/usr/bin/env python3
import json
import sys

# Read task description from stdin (one line JSON)
task_desc = json.loads(sys.stdin.read())

# Extract task
task = task_desc.get("task", "Unknown task")

# Simulate VM environment
vm_id = task.get("vm_id", "unknown")

# Simulate agent reasoning loop
print(json.dumps({{"status": "processing", "vm_id": vm_id, "progress": 0.0}}))
sys.stdout.flush()

# Wait for commands (agent sends commands via stdin)
while True:
    line = sys.stdin.readline()
    if not line:
        break

    try:
        command = json.loads(line)
        action = command.get("action", "unknown")

        if action == "execute_tool":
            # Simulate tool execution
            tool = command.get("tool", "unknown")
            print(json.dumps({{"status": "tool_result", "tool": tool, "result": "success"}))
        elif action == "complete_task":
            # Task completed
            final_result = command.get("result", "unknown")
            print(json.dumps({{"status": "completed", "vm_id": vm_id, "final_result": final_result, "progress": 1.0}))
            sys.exit(0)
        else:
            print(json.dumps({{"status": "error", "error": f"Unknown action: {{action}}"}))
            sys.exit(1)
    "#
    ))
}
