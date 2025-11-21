import { save, open } from '@tauri-apps/plugin-dialog';

export interface UseFileDialogReturn {
  /** Open a save file dialog and return the selected path */
  saveFile: (options?: SaveFileOptions) => Promise<string | null>;
  /** Open an open file dialog and return the selected path */
  openFile: (options?: OpenFileOptions) => Promise<string | null>;
}

export interface SaveFileOptions {
  /** Default file name */
  defaultPath?: string;
  /** File filters */
  filters?: FileFilter[];
  /** Dialog title */
  title?: string;
}

export interface OpenFileOptions {
  /** File filters */
  filters?: FileFilter[];
  /** Dialog title */
  title?: string;
  /** Allow multiple file selection */
  multiple?: boolean;
}

export interface FileFilter {
  /** Filter name (e.g., "JSON Files") */
  name: string;
  /** File extensions (e.g., ["json"]) */
  extensions: string[];
}

/**
 * Hook for file dialog operations using Tauri's dialog plugin.
 */
export function useFileDialog(): UseFileDialogReturn {
  const saveFile = async (options?: SaveFileOptions): Promise<string | null> => {
    try {
      const filePath = await save({
        defaultPath: options?.defaultPath,
        filters: options?.filters,
        title: options?.title,
      });
      return filePath;
    } catch (error) {
      console.error('Save file dialog error:', error);
      return null;
    }
  };

  const openFile = async (options?: OpenFileOptions): Promise<string | null> => {
    try {
      const filePath = await open({
        filters: options?.filters,
        title: options?.title,
        multiple: options?.multiple ?? false,
      });
      
      // open() returns string | string[] | null
      if (Array.isArray(filePath)) {
        return filePath[0] ?? null;
      }
      return filePath;
    } catch (error) {
      console.error('Open file dialog error:', error);
      return null;
    }
  };

  return {
    saveFile,
    openFile,
  };
}
