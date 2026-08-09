import type { AgentWorkspace } from "../../../runtime/agent-workspace/src/index.js";

export interface DriveCommand {
  id: string;
  label: string;
  capability: "read" | "write" | "execute" | "network" | "wallet";
  run(workspace: AgentWorkspace): Promise<void>;
}

export class DriveCommandRegistry {
  private readonly commands = new Map<string, DriveCommand>();

  register(command: DriveCommand): void {
    if (this.commands.has(command.id)) {
      throw new Error(`Duplicate drive command: ${command.id}`);
    }
    this.commands.set(command.id, command);
  }

  list(): readonly DriveCommand[] {
    return [...this.commands.values()];
  }

  async execute(id: string, workspace: AgentWorkspace): Promise<void> {
    const command = this.commands.get(id);
    if (!command) throw new Error(`Unknown drive command: ${id}`);
    workspace.require(command.capability);
    await command.run(workspace);
    await workspace.record("workspace.execute", id);
  }
}

export function createDefaultRegistry(): DriveCommandRegistry {
  const registry = new DriveCommandRegistry();
  registry.register({
    id: "workspace.inspect",
    label: "Inspect workspace",
    capability: "read",
    run: async () => undefined,
  });
  registry.register({
    id: "workspace.sync",
    label: "Sync workspace",
    capability: "write",
    run: async () => undefined,
  });
  return registry;
}
