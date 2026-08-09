export type WorkspaceCapability =
  | "read"
  | "write"
  | "execute"
  | "network"
  | "wallet";

export interface WorkspaceManifest {
  id: string;
  agentId: string;
  capabilities: readonly WorkspaceCapability[];
  createdAt: string;
}

export interface WorkspaceEvent {
  type: "workspace.created" | "workspace.read" | "workspace.write" | "workspace.execute";
  workspaceId: string;
  agentId: string;
  path?: string;
  timestamp: string;
}

export interface WorkspaceStore {
  create(manifest: WorkspaceManifest): Promise<void>;
  get(id: string): Promise<WorkspaceManifest | undefined>;
  appendEvent(event: WorkspaceEvent): Promise<void>;
}

export class AgentWorkspace {
  constructor(
    private readonly manifest: WorkspaceManifest,
    private readonly store: WorkspaceStore,
  ) {}

  get id(): string {
    return this.manifest.id;
  }

  get agentId(): string {
    return this.manifest.agentId;
  }

  can(capability: WorkspaceCapability): boolean {
    return this.manifest.capabilities.includes(capability);
  }

  async record(type: WorkspaceEvent["type"], path?: string): Promise<void> {
    await this.store.appendEvent({
      type,
      workspaceId: this.manifest.id,
      agentId: this.manifest.agentId,
      ...(path === undefined ? {} : { path }),
      timestamp: new Date().toISOString(),
    });
  }

  require(capability: WorkspaceCapability): void {
    if (!this.can(capability)) {
      throw new Error(`Workspace capability denied: ${capability}`);
    }
  }
}
