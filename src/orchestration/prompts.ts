import fs from 'fs';
import path from 'path';

interface PromptContext {
    role: 'planner' | 'executor' | 'tester';
    agentName: string;
    projectName?: string;
    projectDescription?: string;
    projectPath: string;
}

/**
 * Build a system prompt that wraps the user's input with context
 */
export const buildSystemPrompt = (userPrompt: string, context: PromptContext): string => {
    const { role, agentName, projectName, projectDescription, projectPath } = context;

    // Get project structure
    const projectStructure = getProjectStructure(projectPath);

    // Build role-specific system instructions
    const roleInstructions = getRoleInstructions(role, projectPath);

    // Combine everything into a wrapped prompt
    return `${roleInstructions}

## Project Context
${projectName ? `**Project:** ${projectName}` : ''}
${projectDescription ? `**Description:** ${projectDescription}` : ''}
**Working Directory:** ${projectPath}

## Project Structure
\`\`\`
${projectStructure}
\`\`\`

## User Request
${userPrompt}

---

Remember to follow your role as ${agentName} (${role}) and provide clear, actionable responses.`;
};

/**
 * Get role-specific instructions
 * Loads from .chakravarti/prompts/{role}.md if available, otherwise uses defaults
 */
function getRoleInstructions(role: 'planner' | 'executor' | 'tester', projectPath?: string): string {
    // Try to load custom prompts from project
    if (projectPath) {
        const customPromptPath = path.join(projectPath, '.chakravarti', 'prompts', `${role}.md`);
        if (fs.existsSync(customPromptPath)) {
            try {
                const content = fs.readFileSync(customPromptPath, 'utf8');
                // Return the entire file content as instructions
                return content.trim();
            } catch (error) {
                // Fall through to defaults
            }
        }
    }

    // Default instructions
    switch (role) {
        case 'planner':
            return `# You are the Planner Agent

Your responsibilities:
- Collaborate with the user to understand and refine requirements
- Break down features into actionable tasks with story points
- Create sprint plans in markdown format
- Assign tasks to executor agents
- Mediate feedback from the tester
- Resolve merge conflicts and coordinate the team

When creating tasks:
- Use clear, concise titles
- Provide detailed descriptions
- Assign appropriate story points (1, 2, 3, 5, 8, 13)
- Identify dependencies between tasks
- Specify which executor should handle each task`;

        case 'executor':
            return `# You are an Executor Agent

Your responsibilities:
- Implement features according to task specifications
- Write clean, maintainable code
- Follow best practices and coding standards
- Create commits with clear messages
- Work on your assigned branch
- Communicate progress and blockers

When implementing:
- Focus on the specific task assigned to you
- Write self-documenting code
- Consider edge cases and error handling
- Keep changes focused and atomic`;

        case 'tester':
            return `# You are the Tester Agent

Your responsibilities:
- Write comprehensive tests for implementations
- Verify that features work as specified
- Identify bugs and edge cases
- Report issues to the planner with clear reproduction steps
- Ensure code quality and coverage

When testing:
- Write unit tests, integration tests, and e2e tests as appropriate
- Test both happy paths and error cases
- Verify edge cases and boundary conditions
- Provide clear, actionable feedback on issues found`;

        default:
            return '# You are an AI Agent\n\nHelp the user with their request.';
    }
}

/**
 * Get a simplified project structure
 */
export function getProjectStructure(projectPath: string): string {
    try {
        // Check if .chakravarti exists
        const chakravartiPath = path.join(projectPath, '.chakravarti');
        const hasChakravarti = fs.existsSync(chakravartiPath);

        // Get basic directory listing
        const items = fs.readdirSync(projectPath, { withFileTypes: true });

        const structure: string[] = [];

        // Add key files/directories
        for (const item of items) {
            // Skip node_modules, .git, and other common ignores
            if (['node_modules', '.git', 'dist', 'build', '.next'].includes(item.name)) {
                structure.push(`${item.name}/  (hidden)`);
                continue;
            }

            if (item.isDirectory()) {
                structure.push(`${item.name}/`);
            } else {
                structure.push(item.name);
            }
        }

        // Add chakravarti info if it exists
        if (hasChakravarti) {
            structure.push('');
            structure.push('.chakravarti/');
            const sprintsPath = path.join(chakravartiPath, 'sprints');
            if (fs.existsSync(sprintsPath)) {
                const sprints = fs.readdirSync(sprintsPath);
                sprints.forEach(sprint => {
                    structure.push(`  sprints/${sprint}`);
                });
            }
        }

        return structure.join('\n');
    } catch (error) {
        return '(Unable to read project structure)';
    }
}

/**
 * Get prompts for the spec planning phases (YOLO mode)
 */
export function getSpecPrompts(phase: 'specify' | 'plan' | 'tasks'): string {
    switch (phase) {
        case 'specify':
            return `# Phase 1: Specification (YOLO Mode)

Your goal is to draft a comprehensive Product Requirement Document (PRD) based on the user's input.
You must Populate the provided \`01-spec.md\` structure.

**CRITICAL INSTRUCTIONS:**
1. **Be decisive**: Do not ask the user for more info. Make reasonable assumptions to fill in gaps.
2. **Mark Uncertainty**: If you make a significant assumption that might be wrong, mark it with \`[C]\` (Clarification needed).
   - Example: "The app will use Google Login [C]."
   - Example: "Data retention is 30 days [C]."
3. **Completeness**: Fill out every section of the template.

You are acting as a Product Manager.`;

        case 'plan':
            return `# Phase 2: Technical Plan (YOLO Mode)

Your goal is to design the technical architecture and implementation details based on the provided Specification.
You must populate the provided \`02-plan.md\` structure.

**CRITICAL INSTRUCTIONS:**
1. **Be prescriptive**: Choose specific libraries, database schemas, and API signatures.
2. **Mark Uncertainty**: Use \`[C]\` for technical decisions that depend on unverified external factors.
3. **Alignment**: Ensure the plan strictly satisfies the requirements in the Specification.

You are acting as a System Architect.`;

        case 'tasks':
            return `# Phase 3: Task Breakdown (YOLO Mode)

Your goal is to convert the Technical Plan into a list of atomic, actionable tasks.
You must populate the provided \`03-tasks.md\` structure.

**CRITICAL INSTRUCTIONS:**
1. **Atomicity**: Each task should be small enough for an agent to complete in one session.
2. **Dependencies**: listing tasks in logical execution order.
3. **Clarity**: Use clear acceptance criteria for each task.

You are acting as an Engineering Lead.`;

        default:
            return '';
    }
}
