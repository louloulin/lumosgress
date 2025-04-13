#!/usr/bin/env node

import { Command } from 'commander';
import * as readline from 'readline';
import chalk from 'chalk';
import { ProksiClient } from './client';
import { CompletionRequest, Message } from './types';
import dotenv from 'dotenv';
import ora from 'ora';
import { writeFileSync, readFileSync } from 'fs';

// Load environment variables
dotenv.config();

const program = new Command();

// Set up the program metadata
program
  .name('proksi')
  .description('Proksi AI Gateway CLI')
  .version('0.1.0');

// Helper to get client
function getClient() {
  const baseUrl = process.env.PROKSI_API_URL;
  const apiKey = process.env.PROKSI_API_KEY;

  if (!baseUrl) {
    console.error(chalk.red('Error: PROKSI_API_URL environment variable is not set'));
    process.exit(1);
  }

  return new ProksiClient({
    baseUrl,
    apiKey,
  });
}

// Helper to load JSON file
function loadJsonFile(filePath: string): any {
  try {
    const content = readFileSync(filePath, 'utf-8');
    return JSON.parse(content);
  } catch (error) {
    throw new Error(`Failed to load file: ${error instanceof Error ? error.message : String(error)}`);
  }
}

// Completion command
program
  .command('completion')
  .description('Send a completion request to an LLM provider')
  .option('-p, --provider <provider>', 'LLM provider to use')
  .option('-m, --model <model>', 'Model to use', 'gpt-4')
  .option('-t, --temperature <temperature>', 'Temperature for response generation', '0.7')
  .option('-s, --system <system>', 'System message to use')
  .option('-o, --output <file>', 'Save response to a file')
  .argument('<prompt>', 'The user prompt to send')
  .action(async (prompt, options) => {
    const client = getClient();
    const spinner = ora('Sending request...').start();

    try {
      const messages: Message[] = [
        ...(options.system ? [{ role: 'system' as const, content: options.system }] : []),
        { role: 'user' as const, content: prompt }
      ];

      const request: CompletionRequest = {
        provider: options.provider,
        model: options.model,
        messages,
        temperature: parseFloat(options.temperature),
      };

      const response = await client.completion(request);
      spinner.succeed('Response received');
      
      console.log(chalk.green('\nResponse:'));
      console.log(chalk.white(response.message.content));
      console.log(chalk.dim(`\nModel: ${response.model}`));
      console.log(chalk.dim(`Tokens used: ${response.usage.total_tokens}`));

      if (options.output) {
        writeFileSync(options.output, response.message.content);
        console.log(chalk.green(`\nResponse saved to ${options.output}`));
      }
    } catch (error) {
      spinner.fail('Request failed');
      console.error(chalk.red(`Error: ${error instanceof Error ? error.message : String(error)}`));
    }
  });

// Stream completion command
program
  .command('stream')
  .description('Stream a completion from an LLM provider')
  .option('-p, --provider <provider>', 'LLM provider to use')
  .option('-m, --model <model>', 'Model to use', 'gpt-4')
  .option('-t, --temperature <temperature>', 'Temperature for response generation', '0.7')
  .option('-s, --system <system>', 'System message to use')
  .option('-o, --output <file>', 'Save complete response to a file')
  .argument('<prompt>', 'The user prompt to send')
  .action(async (prompt, options) => {
    const client = getClient();
    const spinner = ora('Connecting stream...').start();
    let output = '';

    try {
      const messages: Message[] = [
        ...(options.system ? [{ role: 'system' as const, content: options.system }] : []),
        { role: 'user' as const, content: prompt }
      ];

      const request: CompletionRequest = {
        provider: options.provider,
        model: options.model,
        messages,
        temperature: parseFloat(options.temperature),
      };

      spinner.succeed('Stream connected');
      console.log(chalk.green('\nResponse:'));
      
      await client.streamCompletion(
        request,
        (chunk) => {
          if (chunk.delta.content) {
            process.stdout.write(chunk.delta.content);
            output += chunk.delta.content;
          }
        },
        (error) => {
          console.error(chalk.red(`\nError: ${error.message}`));
        },
        () => {
          console.log(chalk.dim('\n\nStream completed'));
          
          if (options.output) {
            writeFileSync(options.output, output);
            console.log(chalk.green(`Response saved to ${options.output}`));
          }
        }
      );
    } catch (error) {
      spinner.fail('Stream connection failed');
      console.error(chalk.red(`Error: ${error instanceof Error ? error.message : String(error)}`));
    }
  });

// Vector upsert command
program
  .command('vector:upsert')
  .description('Upsert vectors into a vector database')
  .option('-n, --namespace <namespace>', 'Vector namespace', 'default')
  .option('-i, --id <id>', 'Vector ID', String(Date.now()))
  .option('-m, --metadata <json>', 'Vector metadata as JSON string', '{}')
  .option('-f, --file <file>', 'File containing vector values (JSON array) - alternative to inline values')
  .argument('[values...]', 'Vector values (space-separated numbers) - not needed if using --file')
  .action(async (values: string[], options) => {
    const client = getClient();
    const spinner = ora('Upserting vectors...').start();

    try {
      let vectorValues: number[];
      
      if (options.file) {
        try {
          const fileContent = loadJsonFile(options.file);
          if (!Array.isArray(fileContent)) {
            throw new Error('File must contain a JSON array of numbers');
          }
          vectorValues = fileContent;
        } catch (e) {
          spinner.fail('Failed to load vector file');
          console.error(chalk.red(`Error: ${e instanceof Error ? e.message : String(e)}`));
          return;
        }
      } else if (values.length > 0) {
        vectorValues = values.map((v: string) => parseFloat(v));
      } else {
        spinner.fail('No vector values provided');
        console.error(chalk.red('Error: Please provide vector values either inline or via file'));
        return;
      }

      const metadata = JSON.parse(options.metadata);
      
      await client.upsertVectors({
        namespace: options.namespace,
        vectors: [
          {
            id: options.id,
            values: vectorValues,
            metadata
          }
        ]
      });
      
      spinner.succeed('Vectors upserted successfully');
    } catch (error) {
      spinner.fail('Upsert failed');
      console.error(chalk.red(`Error: ${error instanceof Error ? error.message : String(error)}`));
    }
  });

// Vector search command
program
  .command('vector:search')
  .description('Search for similar vectors in a vector database')
  .option('-n, --namespace <namespace>', 'Vector namespace', 'default')
  .option('-k, --top-k <number>', 'Number of results to return', '5')
  .option('-f, --filter <json>', 'Filter as JSON string', '{}')
  .option('--file <file>', 'File containing query vector (JSON array) - alternative to inline values')
  .argument('[values...]', 'Query vector values (space-separated numbers) - not needed if using --file')
  .action(async (values: string[], options) => {
    const client = getClient();
    const spinner = ora('Searching vectors...').start();

    try {
      let queryVector: number[];
      
      if (options.file) {
        try {
          const fileContent = loadJsonFile(options.file);
          if (!Array.isArray(fileContent)) {
            throw new Error('File must contain a JSON array of numbers');
          }
          queryVector = fileContent;
        } catch (e) {
          spinner.fail('Failed to load vector file');
          console.error(chalk.red(`Error: ${e instanceof Error ? e.message : String(e)}`));
          return;
        }
      } else if (values.length > 0) {
        queryVector = values.map((v: string) => parseFloat(v));
      } else {
        spinner.fail('No query vector values provided');
        console.error(chalk.red('Error: Please provide query vector values either inline or via file'));
        return;
      }

      const filter = JSON.parse(options.filter);
      const topK = parseInt(options.topK);
      
      const results = await client.searchVectors({
        namespace: options.namespace,
        query_vector: queryVector,
        top_k: topK,
        filter
      });
      
      spinner.succeed('Search completed');
      
      console.log(chalk.green(`\nFound ${results.results.length} results in namespace "${results.namespace}":`));
      results.results.forEach((result, i) => {
        console.log(chalk.bold(`\n#${i+1} - ID: ${result.id} (Score: ${result.score.toFixed(4)})`));
        if (result.metadata) {
          console.log(chalk.dim('Metadata:'));
          console.log(chalk.dim(JSON.stringify(result.metadata, null, 2)));
        }
      });
    } catch (error) {
      spinner.fail('Search failed');
      console.error(chalk.red(`Error: ${error instanceof Error ? error.message : String(error)}`));
    }
  });

// Vector delete command
program
  .command('vector:delete')
  .description('Delete vectors from a vector database')
  .option('-n, --namespace <namespace>', 'Vector namespace', 'default')
  .argument('<ids...>', 'Vector IDs to delete')
  .action(async (ids, options) => {
    const client = getClient();
    const spinner = ora(`Deleting ${ids.length} vectors...`).start();

    try {
      await client.deleteVectors({
        namespace: options.namespace,
        ids
      });
      
      spinner.succeed(`Deleted ${ids.length} vectors successfully`);
    } catch (error) {
      spinner.fail('Deletion failed');
      console.error(chalk.red(`Error: ${error instanceof Error ? error.message : String(error)}`));
    }
  });

// Interactive mode
program
  .command('chat')
  .description('Start an interactive chat session with an LLM')
  .option('-p, --provider <provider>', 'LLM provider to use')
  .option('-m, --model <model>', 'Model to use', 'gpt-4')
  .option('-t, --temperature <temperature>', 'Temperature for response generation', '0.7')
  .option('-s, --system <system>', 'System message to use', 'You are a helpful AI assistant.')
  .action((options) => {
    const client = getClient();
    const messages: Message[] = [
      { role: 'system' as const, content: options.system }
    ];
    
    console.log(chalk.green('Starting interactive chat session...'));
    console.log(chalk.green(`Provider: ${options.provider || 'default'}`));
    console.log(chalk.green(`Model: ${options.model}`));
    console.log(chalk.green('Type "exit" or press Ctrl+C to end the session\n'));
    
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: chalk.blue('You: ')
    });
    
    rl.prompt();
    
    rl.on('line', async (line) => {
      const input = line.trim();
      if (input.toLowerCase() === 'exit') {
        rl.close();
        return;
      }
      
      messages.push({ role: 'user' as const, content: input });
      const spinner = ora('Thinking...').start();
      
      try {
        const request: CompletionRequest = {
          provider: options.provider,
          model: options.model,
          messages: [...messages],
          temperature: parseFloat(options.temperature),
        };
        
        const response = await client.completion(request);
        spinner.stop();
        console.log(chalk.green('\nAI: ') + response.message.content + '\n');
        
        messages.push(response.message);
      } catch (error) {
        spinner.fail('Request failed');
        console.error(chalk.red(`Error: ${error instanceof Error ? error.message : String(error)}`));
      }
      
      rl.prompt();
    }).on('close', () => {
      console.log(chalk.green('\nChat session ended'));
      process.exit(0);
    });
  });

// Parse command line arguments
program.parse(process.argv);

// If no arguments are provided, show help
if (!process.argv.slice(2).length) {
  program.outputHelp();
} 