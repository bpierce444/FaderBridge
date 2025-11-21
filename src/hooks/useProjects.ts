/**
 * React hook for project management
 * Provides CRUD operations for projects, devices, and mappings
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  Project,
  CreateProjectRequest,
  UpdateProjectRequest,
  Device,
  CreateDeviceRequest,
  Mapping,
  CreateMappingRequest,
  UpdateMappingRequest,
} from '../types/projects';

export interface UseProjectsReturn {
  // Project state
  projects: Project[];
  activeProject: Project | null;
  recentProjects: Project[];
  loading: boolean;
  error: string | null;

  // Project operations
  createProject: (req: CreateProjectRequest) => Promise<Project>;
  updateProject: (req: UpdateProjectRequest) => Promise<Project>;
  deleteProject: (id: number) => Promise<void>;
  setActiveProject: (id: number) => Promise<void>;
  refreshProjects: () => Promise<void>;
  refreshRecentProjects: (limit?: number) => Promise<void>;

  // Device operations
  getDevices: (projectId: number) => Promise<Device[]>;
  createDevice: (req: CreateDeviceRequest) => Promise<Device>;
  deleteDevice: (id: number) => Promise<void>;

  // Mapping operations
  getMappings: (projectId: number) => Promise<Mapping[]>;
  createMapping: (req: CreateMappingRequest) => Promise<Mapping>;
  updateMapping: (req: UpdateMappingRequest) => Promise<Mapping>;
  deleteMapping: (id: number) => Promise<void>;

  // Export/Import operations
  exportProject: (id: number) => Promise<string>;
  exportProjectToFile: (id: number, filePath: string) => Promise<void>;
  importProject: (json: string) => Promise<number>;
  importProjectFromFile: (filePath: string) => Promise<number>;
}

export function useProjects(): UseProjectsReturn {
  const [projects, setProjects] = useState<Project[]>([]);
  const [activeProject, setActiveProjectState] = useState<Project | null>(null);
  const [recentProjects, setRecentProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch all projects
  const refreshProjects = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const allProjects = await invoke<Project[]>('get_all_projects');
      setProjects(allProjects);

      // Update active project
      const active = await invoke<Project | null>('get_active_project');
      setActiveProjectState(active);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      console.error('Failed to refresh projects:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Fetch recent projects
  const refreshRecentProjects = useCallback(async (limit: number = 10) => {
    try {
      const recent = await invoke<Project[]>('get_recent_projects', { limit });
      setRecentProjects(recent);
    } catch (err) {
      console.error('Failed to refresh recent projects:', err);
    }
  }, []);

  // Create a new project
  const createProject = useCallback(
    async (req: CreateProjectRequest): Promise<Project> => {
      setError(null);
      try {
        const project = await invoke<Project>('create_project', {
          name: req.name,
          description: req.description,
        });
        await refreshProjects();
        return project;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        setError(errorMsg);
        throw err;
      }
    },
    [refreshProjects]
  );

  // Update a project
  const updateProject = useCallback(
    async (req: UpdateProjectRequest): Promise<Project> => {
      setError(null);
      try {
        const project = await invoke<Project>('update_project', {
          id: req.id,
          name: req.name,
          description: req.description,
        });
        await refreshProjects();
        return project;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        setError(errorMsg);
        throw err;
      }
    },
    [refreshProjects]
  );

  // Delete a project
  const deleteProject = useCallback(
    async (id: number): Promise<void> => {
      setError(null);
      try {
        await invoke('delete_project', { id });
        await refreshProjects();
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        setError(errorMsg);
        throw err;
      }
    },
    [refreshProjects]
  );

  // Set active project
  const setActiveProject = useCallback(
    async (id: number): Promise<void> => {
      setError(null);
      try {
        await invoke('set_active_project', { id });
        await refreshProjects();
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        setError(errorMsg);
        throw err;
      }
    },
    [refreshProjects]
  );

  // Get devices for a project
  const getDevices = useCallback(async (projectId: number): Promise<Device[]> => {
    try {
      return await invoke<Device[]>('get_devices_by_project', { projectId });
    } catch (err) {
      console.error('Failed to get devices:', err);
      throw err;
    }
  }, []);

  // Create a device
  const createDevice = useCallback(async (req: CreateDeviceRequest): Promise<Device> => {
    try {
      return await invoke<Device>('create_device', { req });
    } catch (err) {
      console.error('Failed to create device:', err);
      throw err;
    }
  }, []);

  // Delete a device
  const deleteDevice = useCallback(async (id: number): Promise<void> => {
    try {
      await invoke('delete_device', { id });
    } catch (err) {
      console.error('Failed to delete device:', err);
      throw err;
    }
  }, []);

  // Get mappings for a project
  const getMappings = useCallback(async (projectId: number): Promise<Mapping[]> => {
    try {
      return await invoke<Mapping[]>('get_mappings_by_project', { projectId });
    } catch (err) {
      console.error('Failed to get mappings:', err);
      throw err;
    }
  }, []);

  // Create a mapping
  const createMapping = useCallback(async (req: CreateMappingRequest): Promise<Mapping> => {
    try {
      return await invoke<Mapping>('create_mapping', { req });
    } catch (err) {
      console.error('Failed to create mapping:', err);
      throw err;
    }
  }, []);

  // Update a mapping
  const updateMapping = useCallback(async (req: UpdateMappingRequest): Promise<Mapping> => {
    try {
      return await invoke<Mapping>('update_mapping', { req });
    } catch (err) {
      console.error('Failed to update mapping:', err);
      throw err;
    }
  }, []);

  // Delete a mapping
  const deleteMapping = useCallback(async (id: number): Promise<void> => {
    try {
      await invoke('delete_mapping', { id });
    } catch (err) {
      console.error('Failed to delete mapping:', err);
      throw err;
    }
  }, []);

  // Export project to JSON
  const exportProject = useCallback(async (id: number): Promise<string> => {
    try {
      return await invoke<string>('export_project', { id });
    } catch (err) {
      console.error('Failed to export project:', err);
      throw err;
    }
  }, []);

  // Export project to file
  const exportProjectToFile = useCallback(
    async (id: number, filePath: string): Promise<void> => {
      try {
        await invoke('export_project_to_file', { id, filePath });
      } catch (err) {
        console.error('Failed to export project to file:', err);
        throw err;
      }
    },
    []
  );

  // Import project from JSON
  const importProject = useCallback(
    async (json: string): Promise<number> => {
      try {
        const projectId = await invoke<number>('import_project', { json });
        await refreshProjects();
        return projectId;
      } catch (err) {
        console.error('Failed to import project:', err);
        throw err;
      }
    },
    [refreshProjects]
  );

  // Import project from file
  const importProjectFromFile = useCallback(
    async (filePath: string): Promise<number> => {
      try {
        const projectId = await invoke<number>('import_project_from_file', { filePath });
        await refreshProjects();
        return projectId;
      } catch (err) {
        console.error('Failed to import project from file:', err);
        throw err;
      }
    },
    [refreshProjects]
  );

  // Load projects on mount
  useEffect(() => {
    refreshProjects();
    refreshRecentProjects();
  }, [refreshProjects, refreshRecentProjects]);

  return {
    projects,
    activeProject,
    recentProjects,
    loading,
    error,
    createProject,
    updateProject,
    deleteProject,
    setActiveProject,
    refreshProjects,
    refreshRecentProjects,
    getDevices,
    createDevice,
    deleteDevice,
    getMappings,
    createMapping,
    updateMapping,
    deleteMapping,
    exportProject,
    exportProjectToFile,
    importProject,
    importProjectFromFile,
  };
}
