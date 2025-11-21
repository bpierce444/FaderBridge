/**
 * Project Manager Component
 * Provides UI for creating, loading, and managing projects
 */

import React, { useState } from 'react';
import { useProjects } from '../hooks/useProjects';
import type { Project } from '../types/projects';

export const ProjectManager: React.FC = () => {
  const {
    projects,
    activeProject,
    recentProjects,
    loading,
    error,
    createProject,
    setActiveProject,
    deleteProject,
    exportProjectToFile,
    importProjectFromFile,
  } = useProjects();

  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [newProjectDescription, setNewProjectDescription] = useState('');

  const handleCreateProject = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newProjectName.trim()) return;

    try {
      await createProject({
        name: newProjectName,
        description: newProjectDescription || undefined,
      });
      setNewProjectName('');
      setNewProjectDescription('');
      setShowCreateDialog(false);
    } catch (err) {
      console.error('Failed to create project:', err);
    }
  };

  const handleLoadProject = async (project: Project) => {
    try {
      await setActiveProject(project.id);
    } catch (err) {
      console.error('Failed to load project:', err);
    }
  };

  const handleDeleteProject = async (project: Project) => {
    if (!confirm(`Are you sure you want to delete "${project.name}"?`)) {
      return;
    }

    try {
      await deleteProject(project.id);
    } catch (err) {
      console.error('Failed to delete project:', err);
    }
  };

  const handleExportProject = async (project: Project) => {
    try {
      // In a real implementation, you'd use a file dialog
      const filePath = `/tmp/${project.name}.json`;
      await exportProjectToFile(project.id, filePath);
      alert(`Project exported to ${filePath}`);
    } catch (err) {
      console.error('Failed to export project:', err);
    }
  };

  const handleImportProject = async () => {
    try {
      // In a real implementation, you'd use a file dialog
      const filePath = prompt('Enter path to project file:');
      if (filePath) {
        await importProjectFromFile(filePath);
        alert('Project imported successfully');
      }
    } catch (err) {
      console.error('Failed to import project:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-400">Loading projects...</div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-white">Projects</h2>
        <div className="flex gap-2">
          <button
            onClick={() => setShowCreateDialog(true)}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors"
          >
            New Project
          </button>
          <button
            onClick={handleImportProject}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md transition-colors"
          >
            Import
          </button>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900/50 border border-red-700 rounded-md text-red-200">
          {error}
        </div>
      )}

      {/* Active Project */}
      {activeProject && (
        <div className="p-4 bg-blue-900/30 border border-blue-700 rounded-md">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-lg font-semibold text-white">Active Project</h3>
              <p className="text-gray-300">{activeProject.name}</p>
              {activeProject.description && (
                <p className="text-sm text-gray-400 mt-1">{activeProject.description}</p>
              )}
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => handleExportProject(activeProject)}
                className="px-3 py-1 bg-gray-700 hover:bg-gray-600 text-white text-sm rounded transition-colors"
              >
                Export
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Recent Projects */}
      {recentProjects.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold text-white mb-3">Recent Projects</h3>
          <div className="grid gap-2">
            {recentProjects.map((project) => (
              <div
                key={project.id}
                className="p-3 bg-gray-800 hover:bg-gray-750 rounded-md transition-colors cursor-pointer"
                onClick={() => handleLoadProject(project)}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-white">{project.name}</p>
                    {project.description && (
                      <p className="text-sm text-gray-400">{project.description}</p>
                    )}
                    <p className="text-xs text-gray-500 mt-1">
                      Last opened: {new Date(project.last_opened_at!).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleExportProject(project);
                      }}
                      className="px-2 py-1 bg-gray-700 hover:bg-gray-600 text-white text-xs rounded transition-colors"
                    >
                      Export
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteProject(project);
                      }}
                      className="px-2 py-1 bg-red-900 hover:bg-red-800 text-white text-xs rounded transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* All Projects */}
      <div>
        <h3 className="text-lg font-semibold text-white mb-3">All Projects</h3>
        {projects.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            No projects yet. Create your first project to get started.
          </div>
        ) : (
          <div className="grid gap-2">
            {projects.map((project) => (
              <div
                key={project.id}
                className="p-3 bg-gray-800 hover:bg-gray-750 rounded-md transition-colors cursor-pointer"
                onClick={() => handleLoadProject(project)}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-white">{project.name}</p>
                    {project.description && (
                      <p className="text-sm text-gray-400">{project.description}</p>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleExportProject(project);
                      }}
                      className="px-2 py-1 bg-gray-700 hover:bg-gray-600 text-white text-xs rounded transition-colors"
                    >
                      Export
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteProject(project);
                      }}
                      className="px-2 py-1 bg-red-900 hover:bg-red-800 text-white text-xs rounded transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Project Dialog */}
      {showCreateDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 p-6 rounded-lg shadow-xl max-w-md w-full">
            <h3 className="text-xl font-bold text-white mb-4">Create New Project</h3>
            <form onSubmit={handleCreateProject} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Project Name
                </label>
                <input
                  type="text"
                  value={newProjectName}
                  onChange={(e) => setNewProjectName(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="My Studio Setup"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">
                  Description (optional)
                </label>
                <textarea
                  value={newProjectDescription}
                  onChange={(e) => setNewProjectDescription(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="Describe your setup..."
                  rows={3}
                />
              </div>
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowCreateDialog(false)}
                  className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors"
                >
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
