export interface TerminalContext {
  viewport: string;
  currentLine: string;
  additionalContext: string;
}

export interface AIResponse {
  suggestions: string[];
  observation: string;
}

export abstract class AISession {
  public abstract query(input: TerminalContext): Promise<AIResponse>;
}
