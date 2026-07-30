interface FileSystemDirectoryHandle {
  entries(): AsyncIterableIterator<
    [string, FileSystemFileHandle | FileSystemDirectoryHandle]
  >;
  getDirectoryHandle(
    name: string,
    options?: { create?: boolean },
  ): Promise<FileSystemDirectoryHandle>;
  removeEntry(name: string, options?: { recursive?: boolean }): Promise<void>;
}

interface Window {
  showDirectoryPicker?(options?: {
    id?: string;
    mode?: "read" | "readwrite";
  }): Promise<FileSystemDirectoryHandle>;
}
