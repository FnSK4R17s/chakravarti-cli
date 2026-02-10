/**
 * @module ProjectSelector
 * @description
 * Full-page project selection screen for Tauri desktop app.
 * Shows on first launch when no project root is configured.
 * Allows selecting a folder via native dialog or from recent projects.
 *
 * @context
 * Used as an entry gate in Tauri mode - if no project_root is set,
 * this screen is shown before the main app. After selection, the
 * app reloads with the new project.
 *
 * @dependencies
 * - @tauri-apps/api: For invoke and native dialog
 * - shadcn/ui: Card, Button components
 * - lucide-react: Icons
 */

import { useState, useEffect } from 'react';
import { FolderOpen, Clock, Folder, RefreshCw, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

// ===== Types =====

/**
 * Information about a recent project for display in the selector.
 */
interface ProjectInfo {
    /** Full filesystem path to the project */
    path: string;
    /** Display name of the project (folder name) */
    name: string;
    /** Whether the project folder still exists on disk */
    exists: boolean;
}

/**
 * Props for the ProjectSelector component.
 */
interface ProjectSelectorProps {
    /** Callback fired after a project is selected and saved */
    onProjectSelected?: () => void;
}

// ===== Main Component =====

export default function ProjectSelector({ onProjectSelected }: ProjectSelectorProps) {
    // State
    const [recentProjects, setRecentProjects] = useState<ProjectInfo[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [isSelecting, setIsSelecting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Load recent projects on mount
    useEffect(() => {
        const loadRecentProjects = async () => {
            try {
                const { invoke } = await import('@tauri-apps/api/core');
                const projects = await invoke<ProjectInfo[]>('get_recent_projects');
                setRecentProjects(projects);
            } catch (e) {
                console.error('Failed to load recent projects:', e);
            } finally {
                setIsLoading(false);
            }
        };

        loadRecentProjects();
    }, []);

    // Handle folder selection via native dialog
    const handleChooseFolder = async () => {
        setIsSelecting(true);
        setError(null);

        try {
            const { invoke } = await import('@tauri-apps/api/core');

            // Open native folder picker
            const selectedPath = await invoke<string | null>('open_project_dialog');

            if (selectedPath) {
                // Save the selected project
                await invoke('set_project_root', { path: selectedPath });

                // Restart the entire Tauri app to apply new project root
                try {
                    const { relaunch } = await import('@tauri-apps/plugin-process');
                    await relaunch();
                } catch {
                    // Fallback: notify parent if relaunch not available
                    if (onProjectSelected) {
                        onProjectSelected();
                    } else {
                        window.location.reload();
                    }
                }
            }
        } catch (e) {
            setError(e instanceof Error ? e.message : String(e));
        } finally {
            setIsSelecting(false);
        }
    };

    // Handle selecting a recent project
    const handleSelectRecent = async (project: ProjectInfo) => {
        if (!project.exists) {
            setError(`Project folder not found: ${project.path}`);
            return;
        }

        setIsSelecting(true);
        setError(null);

        try {
            const { invoke } = await import('@tauri-apps/api/core');

            // Save the selected project
            await invoke('set_project_root', { path: project.path });

            // Restart the entire Tauri app to apply new project root
            try {
                const { relaunch } = await import('@tauri-apps/plugin-process');
                await relaunch();
            } catch {
                // Fallback: notify parent if relaunch not available
                if (onProjectSelected) {
                    onProjectSelected();
                } else {
                    window.location.reload();
                }
            }
        } catch (e) {
            setError(e instanceof Error ? e.message : String(e));
        } finally {
            setIsSelecting(false);
        }
    };

    return (
        <div className="min-h-screen flex items-center justify-center bg-background p-8">
            <Card className="w-full max-w-lg">
                <CardHeader className="text-center">
                    <div className="mx-auto mb-4 w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center">
                        <FolderOpen className="w-8 h-8 text-primary" />
                    </div>
                    <CardTitle className="text-2xl">Select a Project</CardTitle>
                    <CardDescription>
                        Choose a folder containing your project to get started with Chakravarti
                    </CardDescription>
                </CardHeader>

                <CardContent className="space-y-6">
                    {/* Error display */}
                    {error && (
                        <div className="flex items-center gap-2 p-3 rounded-md bg-destructive/10 text-destructive text-sm">
                            <AlertCircle className="w-4 h-4 flex-shrink-0" />
                            <span>{error}</span>
                        </div>
                    )}

                    {/* Main action button */}
                    <Button
                        className="w-full h-12 text-base"
                        onClick={handleChooseFolder}
                        disabled={isSelecting}
                    >
                        {isSelecting ? (
                            <>
                                <RefreshCw className="w-5 h-5 mr-2 animate-spin" />
                                Opening...
                            </>
                        ) : (
                            <>
                                <FolderOpen className="w-5 h-5 mr-2" />
                                Choose Folder
                            </>
                        )}
                    </Button>

                    {/* Recent projects */}
                    {!isLoading && recentProjects.length > 0 && (
                        <div className="space-y-3">
                            <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                <Clock className="w-4 h-4" />
                                <span>Recent Projects</span>
                            </div>

                            <div className="space-y-2">
                                {recentProjects.map((project, index) => (
                                    <button
                                        key={index}
                                        onClick={() => handleSelectRecent(project)}
                                        disabled={isSelecting}
                                        className={`w-full p-3 rounded-md border text-left transition-colors
                      ${project.exists
                                                ? 'hover:bg-muted/50 cursor-pointer border-border'
                                                : 'opacity-50 cursor-not-allowed border-border/50'
                                            }
                    `}
                                    >
                                        <div className="flex items-center gap-3">
                                            <Folder className={`w-5 h-5 flex-shrink-0 ${project.exists ? 'text-primary' : 'text-muted-foreground'}`} />
                                            <div className="min-w-0 flex-1">
                                                <div className="font-medium truncate">{project.name}</div>
                                                <div className="text-xs text-muted-foreground truncate">
                                                    {project.path}
                                                </div>
                                            </div>
                                            {!project.exists && (
                                                <span className="text-xs text-destructive">Not found</span>
                                            )}
                                        </div>
                                    </button>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Loading state for recent projects */}
                    {isLoading && (
                        <div className="flex items-center justify-center py-4">
                            <RefreshCw className="w-5 h-5 animate-spin text-muted-foreground" />
                        </div>
                    )}

                    {/* Tip */}
                    <p className="text-xs text-center text-muted-foreground">
                        Select a folder containing a Git repository with specs to manage
                    </p>
                </CardContent>
            </Card>
        </div>
    );
}
