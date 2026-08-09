export interface AutomationContext {
  workspaceId: string;
  agentId: string;
  signal: AbortSignal;
}

export interface AutomationModule {
  readonly id: string;
  readonly version: string;
  run(context: AutomationContext): Promise<void>;
}

export class AutomationRegistry {
  private readonly modules = new Map<string, AutomationModule>();

  register(module: AutomationModule): void {
    if (this.modules.has(module.id)) {
      throw new Error(`Duplicate automation module: ${module.id}`);
    }
    this.modules.set(module.id, module);
  }

  get(id: string): AutomationModule {
    const module = this.modules.get(id);
    if (!module) throw new Error(`Automation module not found: ${id}`);
    return module;
  }

  async run(id: string, context: AutomationContext): Promise<void> {
    if (context.signal.aborted) throw new DOMException("Automation cancelled", "AbortError");
    await this.get(id).run(context);
  }
}
