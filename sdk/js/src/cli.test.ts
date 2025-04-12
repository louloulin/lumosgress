import { exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';

// Declare Jest globals to fix linting issues
declare const describe: (name: string, fn: () => void) => void;
declare const test: {
  (name: string, fn: (done?: jest.DoneCallback) => void, timeout?: number): void;
  skip: (name: string, fn: (done?: jest.DoneCallback) => void, timeout?: number) => void;
};
declare const expect: jest.Expect;
declare const fail: (message?: string) => void;

const execAsync = promisify(exec);
const CLI_PATH = path.resolve(__dirname, '..', 'dist', 'cli.js');

// Skip the tests if no environment variables are set
const hasEnvVars = process.env.PROKSI_API_URL && process.env.PROKSI_API_KEY;

describe('CLI', () => {
  // This test can run without actual API calls
  test('should output help information', async () => {
    const { stdout } = await execAsync(`node ${CLI_PATH} --help`);
    expect(stdout).toContain('Usage:');
    expect(stdout).toContain('Options:');
    expect(stdout).toContain('Commands:');
  });

  // These tests require API access, so we conditionally skip them
  (hasEnvVars ? test : test.skip)('should run completion command', async () => {
    const { stdout } = await execAsync(`node ${CLI_PATH} completion "Hello"`);
    expect(stdout).toContain('Response:');
  }, 15000);

  (hasEnvVars ? test : test.skip)('should handle vector operations', async () => {
    // Test vector upsert
    const upsertCmd = `node ${CLI_PATH} vector:upsert --namespace test-ns --id test-id-${Date.now()} 0.1 0.2 0.3 0.4`;
    const upsertResult = await execAsync(upsertCmd);
    expect(upsertResult.stdout).toContain('Vectors upserted successfully');

    // Test vector search
    const searchCmd = `node ${CLI_PATH} vector:search --namespace test-ns 0.1 0.2 0.3 0.4`;
    const searchResult = await execAsync(searchCmd);
    expect(searchResult.stdout).toContain('Search completed');
  }, 30000);

  // Error handling tests
  test('should display error for missing PROKSI_API_URL', async () => {
    // Temporarily unset the environment variable
    const oldUrl = process.env.PROKSI_API_URL;
    process.env.PROKSI_API_URL = '';
    
    try {
      // This command should fail with an error about missing URL
      await execAsync(`node ${CLI_PATH} completion "test"`);
      // If we get here without error, the test should fail
      throw new Error('Should have thrown an error about missing PROKSI_API_URL');
    } catch (error: any) {
      expect(error.stderr).toContain('PROKSI_API_URL environment variable is not set');
    } finally {
      // Restore the environment variable
      process.env.PROKSI_API_URL = oldUrl;
    }
  });
}); 