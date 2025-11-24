/**
 * TopBar component
 * Application header with project management controls
 */

import { useState } from 'react';
import { useProjects } from '../hooks/useProjects';
import { useFileDialog } from '../hooks/useFileDialog';

export interface TopBarProps {
  /** Callback when save is triggered (Cmd+S) */
  onSave?: () => void;
}

/**
 * Top bar with project management controls
 * Includes New, Save, Load, Export/Import functionality
 * 
 * @example
 * ```tsx
 * <TopBar onSave={handleSave} />
 * ```
 */
export function TopBar({ onSave }: TopBarProps) {
  const {
    activeProject,
    createProject,
    exportProjectToFile,
    importProjectFromFile,
  } = useProjects();

  const { saveFile, openFile } = useFileDialog();
  const [showNewProjectDialog, setShowNewProjectDialog] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [newProjectDescription, setNewProjectDescription] = useState('');

  const handleNewProject = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newProjectName.trim()) return;

    try {
      await createProject({
        name: newProjectName,
        description: newProjectDescription || undefined,
      });
      setNewProjectName('');
      setNewProjectDescription('');
      setShowNewProjectDialog(false);
    } catch (err) {
      console.error('Failed to create project:', err);
    }
  };

  const handleExport = async () => {
    if (!activeProject) return;

    try {
      const filePath = await saveFile({
        defaultPath: `${activeProject.name}.json`,
        filters: [{ name: 'FaderBridge Project', extensions: ['json'] }],
      });

      if (filePath) {
        await exportProjectToFile(activeProject.id, filePath);
      }
    } catch (err) {
      console.error('Failed to export project:', err);
    }
  };

  const handleImport = async () => {
    try {
      const filePath = await openFile({
        filters: [{ name: 'FaderBridge Project', extensions: ['json'] }],
      });

      if (filePath) {
        await importProjectFromFile(filePath);
      }
    } catch (err) {
      console.error('Failed to import project:', err);
    }
  };

  const handleSave = () => {
    if (onSave) {
      onSave();
    }
  };

  return (
    <>
      <div className="h-16 px-6 flex items-center justify-between bg-slate-900">
        {/* Left: App Title */}
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-cyan-500">FaderBridge</h1>
          {activeProject && (
            <div className="flex items-center gap-2 text-sm">
              <span className="text-slate-500">•</span>
              <span className="text-slate-300 font-medium">{activeProject.name}</span>
            </div>
          )}
        </div>

        {/* Right: Project Controls */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowNewProjectDialog(true)}
            className="px-3 py-1.5 text-sm bg-slate-800 hover:bg-slate-700 text-white rounded transition-colors"
            aria-label="New project"
          >
            New
          </button>
          <button
            onClick={handleSave}
            disabled={!activeProject}
            className="px-3 py-1.5 text-sm bg-slate-800 hover:bg-slate-700 disabled:bg-slate-900 disabled:text-slate-600 text-white rounded transition-colors"
            aria-label="Save project (Cmd+S)"
            title="Cmd+S"
          >
            Save
          </button>
          <button
            onClick={handleExport}
            disabled={!activeProject}
            className="px-3 py-1.5 text-sm bg-slate-800 hover:bg-slate-700 disabled:bg-slate-900 disabled:text-slate-600 text-white rounded transition-colors"
            aria-label="Export project"
          >
            Export
          </button>
          <button
            onClick={handleImport}
            className="px-3 py-1.5 text-sm bg-slate-800 hover:bg-slate-700 text-white rounded transition-colors"
            aria-label="Import project"
          >
            Import
          </button>
        </div>
      </div>

      {/* New Project Dialog */}
      {showNewProjectDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-slate-800 p-6 rounded-lg shadow-xl max-w-md w-full border border-slate-700">
            <h3 className="text-xl font-bold text-white mb-4">Create New Project</h3>
            <form onSubmit={handleNewProject} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-1">
                  Project Name
                </label>
                <input
                  type="text"
                  value={newProjectName}
                  onChange={(e) => setNewProjectName(e.target.value)}
                  className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded text-white focus:outline-none focus:ring-2 focus:ring-cyan-500"
                  placeholder="My Studio Setup"
                  autoFocus
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-300 mb-1">
                  Description (optional)
                </label>
                <textarea
                  value={newProjectDescription}
                  onChange={(e) => setNewProjectDescription(e.target.value)}
                  className="w-full px-3 py-2 bg-slate-900 border border-slate-700 rounded text-white focus:outline-none focus:ring-2 focus:ring-cyan-500"
                  placeholder="Describe your setup..."
                  rows={3}
                />
              </div>
              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => {
                    setShowNewProjectDialog(false);
                    setNewProjectName('');
                    setNewProjectDescription('');
                  }}
                  className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-cyan-500 hover:bg-cyan-600 text-slate-950 font-semibold rounded transition-colors"
                >
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </>
  );
}
