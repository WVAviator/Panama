import { AIResponse, AISession, TerminalContext } from './AISession';
import { OPENAI_API_KEY } from './apikey';
import OpenAI from 'openai';

const SYSTEM_PROMPT = ` You are an AI expert in using the command line. You are tasked with generating autocomplete suggestions for terminal shell users based on the information provided. You will receive a JSON object with the following fields:

- viewport - a string that contains all the lines that are currently visible in the terminal viewport. Lines are separated by the newline character.
- currentLine - a string that contains the text that the user is currently typing. Your completion suggestions will be appended to this text.
- additionalContext - a string that contains additional information about the current state of the terminal. This can be empty.

Here is an example:

{
  "viewport": "Changes not staged for commit:\n(use \"git add <file>...\" to update what will be committed)\n(use \"git restore <file>...\" to discard changes in working directory)\n      modified:   package.json\n      modified:   src/App.tsx\n   \nUntracked files:\n (use \"git add <file>...\" to include in what will be committed)",
  "currentLine": "gi",
  "additionalContext": ""
}

From this information, you are to generate a JSON object with the following fields:

- suggestions - an array of 1-5 strings that contain the autocomplete suggestions for the current line. Suggestions should take into account any text the user has already typed. The suggestions should be sorted by the most likely to be used first.
- observation - a string containing meta information about what you think the user is currently trying to do.

Please only generate JSON objects with the above fields. You can use the following example response to the above example input:

{
  "suggestions": ["t add .", "t add package.json", "t checkout -b app-modifications"],
  "observation": "The user is trying to add files to the staging area. They probably plan to commit changes soon."
}
`;

export class OpenAISession extends AISession {
  private openai: OpenAI;
  private debounce: NodeJS.Timeout | undefined;

  public constructor() {
    super();
    this.openai = new OpenAI({
      apiKey: OPENAI_API_KEY,
      dangerouslyAllowBrowser: true,
    });
  }

  public async query(input: TerminalContext): Promise<AIResponse> {
    if (this.debounce) {
      clearTimeout(this.debounce);
    }
    return new Promise((resolve, reject) => {
      this.debounce = setTimeout(async () => {
        try {
          resolve(await this.getResponse(input));
        } catch (e) {
          reject(e);
        }
      }, 5000);
    });
  }

  private async getResponse(input: TerminalContext) {
    const completion = await this.openai.chat.completions.create({
      messages: [
        { role: 'system', content: SYSTEM_PROMPT },
        { role: 'user', content: JSON.stringify(input) },
      ],
      model: 'gpt-3.5-turbo',
    });

    const response = completion.choices[0].message.content;
    const parsedResponse = JSON.parse(response || '');

    return {
      suggestions: parsedResponse.suggestions,
      observation: parsedResponse.observation,
    };
  }
}
