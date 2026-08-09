import type { FileSystem, WebContainer } from "@webcontainer/api";

export interface DriveEngine {
  ingest(data: Uint8Array): Promise<string>;
  read(hash: string): Promise<Uint8Array>;
  version(): string;
}

export interface DriveFile {
  hash: string;
  size: number;
}

export class WebContainerDrive {
  constructor(private readonly engine: DriveEngine, private readonly fs: FileSystem) {}

  async putFile(path: string): Promise<DriveFile> {
    const data = await this.fs.readFile(path);
    const hash = await this.engine.ingest(data);
    return { hash, size: data.byteLength };
  }

  async getFile(hash: string, path: string): Promise<void> {
    const data = await this.engine.read(hash);
    await this.fs.writeFile(path, data);
  }
}

export async function mountWebContainer(
  engine: DriveEngine,
  container: WebContainer,
): Promise<WebContainerDrive> {
  return new WebContainerDrive(engine, container.fs);
}

export function assertHash(hash: string): void {
  if (!/^[0-9a-f]{64}$/.test(hash)) {
    throw new Error("Invalid SovereignDrive content hash");
  }
}
