import { invoke } from '@tauri-apps/api/core';

export interface Skill {
  name: string;
  path: string;
  content: string;
}

export class SkillLoader {
  private projectRoot: string;

  constructor(projectRoot: string) {
    this.projectRoot = projectRoot;
  }

  /**
   * Load skills from the filesystem via Tauri IPC.
   */
  async loadSkills(skillPaths: string[]): Promise<Skill[]> {
    const skills: Skill[] = [];

    for (const relPath of skillPaths) {
      try {
        const content = await invoke<string>('read_file_cmd', {
          projectRoot: this.projectRoot,
          path: `${this.projectRoot}/agents/${relPath}`,
        });
        skills.push({
          name: relPath.split('/').pop()?.replace('.md', '') ?? 'unknown',
          path: relPath,
          content,
        });
      } catch (e) {
        console.warn(`[SkillLoader] Failed to load skill ${relPath}:`, e);
      }
    }
    return skills;
  }

  /**
   * Concatenate all skill contents into a single context block
   * for injection into the agent prompt.
   */
  static buildSkillContext(skills: Skill[]): string {
    return skills
      .map(s => `## Skill: ${s.name}\n\n${s.content}`)
      .join('\n\n---\n\n');
  }
}
