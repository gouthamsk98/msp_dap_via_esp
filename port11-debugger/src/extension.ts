// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from "vscode";
import * as cp from "child_process";
import * as path from "path";

// Global variable to track the running process
let runningProcess: cp.ChildProcess | null = null;

// Status bar items
let playStatusBarItem: vscode.StatusBarItem;
let stopStatusBarItem: vscode.StatusBarItem;

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
  // Use the console to output diagnostic information (console.log) and errors (console.error)
  // This line of code will only be executed once when your extension is activated
  console.log(
    'Congratulations, your extension "port11-debugger" is now active!'
  );

  // Create status bar items
  playStatusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100
  );
  playStatusBarItem.command = "port11-debugger.play";
  playStatusBarItem.text = "$(play) Play Debugger";
  playStatusBarItem.tooltip = "Start Port11 Debugger";

  stopStatusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    99
  );
  stopStatusBarItem.command = "port11-debugger.stop";
  stopStatusBarItem.text = "$(stop) Stop Debugger";
  stopStatusBarItem.tooltip = "Stop Port11 Debugger";

  // Function to update status bar visibility
  const updateStatusBar = () => {
    if (runningProcess) {
      playStatusBarItem.hide();
      stopStatusBarItem.show();
    } else {
      playStatusBarItem.show();
      stopStatusBarItem.hide();
    }
  };

  // Initialize status bar state
  updateStatusBar();

  // The command has been defined in the package.json file
  // Now provide the implementation of the command with registerCommand
  // The commandId parameter must match the command field in package.json
  const disposable = vscode.commands.registerCommand(
    "port11-debugger.helloWorld",
    () => {
      // The code you place here will be executed every time your command is executed
      // Display a message box to the user
      vscode.window.showInformationMessage("Hello World from port11_debugger!");
    }
  );

  // Register the play command
  const playCommand = vscode.commands.registerCommand(
    "port11-debugger.play",
    async () => {
      if (runningProcess) {
        vscode.window.showWarningMessage(
          "Debugger is already running. Stop it first."
        );
        return;
      }

      const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
      if (!workspaceFolder) {
        vscode.window.showErrorMessage("No workspace folder found.");
        return;
      }

      // Path to the Rust CLI application
      const rootPath = path.dirname(workspaceFolder.uri.fsPath);
      const cargoTomlPath = path.join(rootPath, "Cargo.toml");

      // Check if we're in the right directory structure
      try {
        await vscode.workspace.fs.stat(vscode.Uri.file(cargoTomlPath));
      } catch {
        vscode.window.showErrorMessage(
          "Could not find Cargo.toml in parent directory."
        );
        return;
      }

      // Show output channel for the debugger
      const outputChannel =
        vscode.window.createOutputChannel("Port11 Debugger");
      outputChannel.show();

      try {
        // Build and run the Rust CLI application
        outputChannel.appendLine("Building and starting Port11 debugger...");

        runningProcess = cp.spawn("cargo", ["run"], {
          cwd: rootPath,
          stdio: ["pipe", "pipe", "pipe"],
        });

        runningProcess.stdout?.on("data", (data) => {
          outputChannel.append(data.toString());
        });

        runningProcess.stderr?.on("data", (data) => {
          outputChannel.append(data.toString());
        });

        runningProcess.on("close", (code) => {
          outputChannel.appendLine(`\nProcess exited with code ${code}`);
          runningProcess = null;
          updateStatusBar();
        });

        runningProcess.on("error", (error) => {
          outputChannel.appendLine(`\nError: ${error.message}`);
          runningProcess = null;
          updateStatusBar();
          vscode.window.showErrorMessage(
            `Failed to start debugger: ${error.message}`
          );
        });

        vscode.window.showInformationMessage("Port11 debugger started.");
        updateStatusBar();
      } catch (error) {
        outputChannel.appendLine(`\nFailed to start: ${error}`);
        vscode.window.showErrorMessage(`Failed to start debugger: ${error}`);
      }
    }
  );

  // Register the stop command
  const stopCommand = vscode.commands.registerCommand(
    "port11-debugger.stop",
    () => {
      if (!runningProcess) {
        vscode.window.showWarningMessage(
          "No debugger process is currently running."
        );
        return;
      }

      try {
        runningProcess.kill("SIGTERM");
        vscode.window.showInformationMessage("Port11 debugger stopped.");
      } catch (error) {
        vscode.window.showErrorMessage(`Failed to stop debugger: ${error}`);
      }

      runningProcess = null;
      updateStatusBar();
    }
  );

  context.subscriptions.push(
    disposable,
    playCommand,
    stopCommand,
    playStatusBarItem,
    stopStatusBarItem
  );
}

// This method is called when your extension is deactivated
export function deactivate() {
  // Clean up any running processes
  if (runningProcess) {
    runningProcess.kill("SIGTERM");
    runningProcess = null;
  }

  // Dispose status bar items
  if (playStatusBarItem) {
    playStatusBarItem.dispose();
  }
  if (stopStatusBarItem) {
    stopStatusBarItem.dispose();
  }
}
