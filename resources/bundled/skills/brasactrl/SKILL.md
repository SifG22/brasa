---
name: brasactrl
description: Control and inspect the currently running local Brasa application with the brasactrl CLI. Use this skill whenever the user asks the agent to manipulate Brasa's own windows, tabs, panes, sessions, input buffer, themes, or UI surfaces; open a file in Brasa; inspect local Brasa state; or explain how to invoke Brasa Control manually.
---

# Brasa Control

Use `{{brasactrl_binary_name}}` to inspect or control the already-running local Brasa application that provided this skill. The command name and wrapper path in this skill are injected for the current Brasa channel, so do not inspect running processes or guess which channel is active.

Prefer `{{brasactrl_binary_name}}` when the requested action changes Brasa itself rather than the user's project or operating system. Examples include creating a Brasa tab, splitting a pane, staging text in Brasa's input, opening Brasa settings, or focusing a Brasa window.

## How to invoke Brasa Control

Brasa Control is bundled into the Brasa application. It is not a separate standalone binary; it is a hidden control mode served by the running Brasa process.

- Command for the current Brasa channel: `{{brasactrl_binary_name}}`
- Bundled wrapper for the current Brasa channel: `{{brasactrl_wrapper_path}}`
- Optional PATH symlink: `/usr/local/bin/{{brasactrl_binary_name}}`

### Ensure the command is available

Before invoking Brasa Control for the first time in a task, prefer the shortest available path and avoid unnecessary setup research:

1. If `command -v {{brasactrl_binary_name}}` succeeds, use `{{brasactrl_binary_name}}` for the rest of the task. Do not inspect the bundled wrapper or verify the symlink unless a later command fails.
2. If `command -v {{brasactrl_binary_name}}` fails, verify that `{{brasactrl_wrapper_path}}` exists and is executable. If it is missing, tell the user that this Brasa build does not contain the expected wrapper and stop.
3. Inspect `/usr/local/bin/{{brasactrl_binary_name}}`. Treat setup as complete only when it is a symlink that resolves to the exact `{{brasactrl_wrapper_path}}` bundled wrapper.
4. If the expected symlink is missing, broken, or points elsewhere, use the `ask_user_question` tool to ask whether the user wants to install it at `/usr/local/bin/{{brasactrl_binary_name}}` pointing to `{{brasactrl_wrapper_path}}`. Offer **Install command** as the recommended option and **Not now** as the alternative. Do not create or replace a symlink without an affirmative response.
5. After approval, create or update only the expected symlink by running `ln -sf "{{brasactrl_wrapper_path}}" "/usr/local/bin/{{brasactrl_binary_name}}"`. Try without elevation first. If macOS permissions prevent the change, run that same command through `osascript` with administrator privileges; never request or expose the user's password directly.
6. Verify the result with `command -v {{brasactrl_binary_name}}`, `readlink /usr/local/bin/{{brasactrl_binary_name}}`, and `{{brasactrl_binary_name}} app version`.

If the user chooses **Not now**, do not create the symlink. Use the bundled wrapper at `{{brasactrl_wrapper_path}}` directly for the current task.

The Brasa UI also exposes **Install Brasa Control CLI command** and **Uninstall Brasa Control CLI command** in the Command Palette and an install control under **Settings > Scripting**.

## Workflow

Always prefer discovering commands from `{{brasactrl_binary_name}}` itself rather than guessing or inventing them. The CLI provides full help and an action catalog that is the authoritative source of truth for what the installed build supports.

### Execute serially and validate results

Run Brasa Control commands serially. Never dispatch multiple `{{brasactrl_binary_name}}` commands through parallel shell-tool calls, even when the commands appear independent. They act on the same running app and may change the active target or the terminal context used to execute and observe later commands. For multi-step requests, prefer one shell-tool call that chains commands sequentially, or issue separate shell-tool calls one at a time.

After an action that creates, activates, navigates, or focuses a window, tab, pane, session, or surface, do not assume the active target is unchanged. Use explicit selectors for later commands when exact targeting matters, or rerun `{{brasactrl_binary_name}} app active` before continuing.

Validate that each result corresponds to the command that was invoked. If output describes a different action, reports an unexpected instance or channel, or otherwise conflicts with the request, stop and rerun `{{brasactrl_binary_name}} instance list` serially before retrying. Do not report success until the requested final state has been verified when a corresponding `list`, `inspect`, or `get` command is available.

### Route by intent

Before discovering commands, route the request to the narrowest matching top-level group:

1. Requests to open, show, view, or toggle a named Brasa UI destination, panel, picker, or settings page use `surface`. Convert natural-language names to kebab case, such as "Brasa Drive" to `brasa-drive` and "code review" to `code-review`. Prefer `surface <name> open` when the requested final state is open. Use `surface list` or `surface help` when the destination or supported verb is unknown. Do not infer an internal action name for a UI destination.
2. Requests about windows, tabs, panes, or sessions use the matching `window`, `tab`, `pane`, or `session` group.
3. Requests to stage or inspect editor input use `input`.
4. Requests to open a file in Brasa use `file`.
5. Requests about themes, appearance, settings, or keybindings use the matching `theme`, `appearance`, `setting`, or `keybinding` group.
6. Use the generic `action` catalog only when no dedicated CLI group matches. Internal or catalog action names are not guaranteed to be reachable as standalone parser commands.

1. Discover running Brasa instances from the current Brasa channel:

   ```sh
   {{brasactrl_binary_name}} instance list
   ```

2. If exactly one same-channel instance is running, commands select it automatically. If multiple same-channel instances are running, select one explicitly with `--instance <instance_id>` or `--pid <pid>`.

3. Discover the exact command and parameters from the routed group instead of guessing. This is the preferred source of truth for the available command surface:

   ```sh
   {{brasactrl_binary_name}} help
   {{brasactrl_binary_name}} <group> help
   {{brasactrl_binary_name}} <group> <command> --help
   ```

   Only when no dedicated group matches, inspect the generic action catalog:

   ```sh
   {{brasactrl_binary_name}} action list
   {{brasactrl_binary_name}} action inspect <action.name>
   ```

4. Inspect the active target chain or list the relevant targets before changing them:

   ```sh
   {{brasactrl_binary_name}} app active
   {{brasactrl_binary_name}} window list
   {{brasactrl_binary_name}} tab list
   {{brasactrl_binary_name}} pane list
   {{brasactrl_binary_name}} session list
   ```

5. Invoke the narrowest action that satisfies the request, then verify the result with the corresponding `list`, `inspect`, or `get` command when useful.

## Common actions

These are frequently used commands that are safe to invoke directly. For less common commands, route by intent and use `{{brasactrl_binary_name}} <group> help` or `{{brasactrl_binary_name}} <group> <command> --help` to discover the exact syntax supported by the running build. Inspect the generic action catalog only when no dedicated group matches.

```sh
# Create and manage tabs and panes
{{brasactrl_binary_name}} tab create
{{brasactrl_binary_name}} tab create --type agent
{{brasactrl_binary_name}} tab rename "server logs"
{{brasactrl_binary_name}} pane split --direction right
{{brasactrl_binary_name}} pane navigate --direction next

# Stage text in Brasa's input without submitting it
{{brasactrl_binary_name}} input insert "git status"
{{brasactrl_binary_name}} input replace "cargo test"

# Open or toggle Brasa UI surfaces
{{brasactrl_binary_name}} surface list
{{brasactrl_binary_name}} surface settings open
{{brasactrl_binary_name}} surface command-palette open --query "theme"
{{brasactrl_binary_name}} surface command-search open
{{brasactrl_binary_name}} surface theme-picker open
{{brasactrl_binary_name}} surface keybindings open
{{brasactrl_binary_name}} surface brasa-drive open
{{brasactrl_binary_name}} surface resource-center toggle
{{brasactrl_binary_name}} surface ai-assistant toggle
{{brasactrl_binary_name}} surface project-explorer open
{{brasactrl_binary_name}} surface global-search open
{{brasactrl_binary_name}} surface conversation-list open
{{brasactrl_binary_name}} surface code-review open
{{brasactrl_binary_name}} surface left-panel toggle
{{brasactrl_binary_name}} surface right-panel toggle
{{brasactrl_binary_name}} surface vertical-tabs open
{{brasactrl_binary_name}} surface agent-management open

# Open a file in Brasa
{{brasactrl_binary_name}} file open ./src/main.rs --line 42

# Inspect and update supported state
{{brasactrl_binary_name}} theme get
{{brasactrl_binary_name}} theme set "Dracula"
{{brasactrl_binary_name}} appearance get
{{brasactrl_binary_name}} setting list
{{brasactrl_binary_name}} keybinding list
```

Add `--output-format json` when structured output is easier to consume:

```sh
{{brasactrl_binary_name}} --output-format json tab list
```

## Targeting

Target selectors can be combined when the action supports their scope:

- Instance: `--instance <instance_id>` or `--pid <pid>`
- Window: `--window <id>`, `--window-index <n>`, or `--window-title <exact-title>`
- Tab: `--tab <id>`, `--tab-index <n>`, or `--tab-title <exact-title>`
- Pane: `--pane <id>` or `--pane-index <n>`
- Session: `--session <id>`

Use IDs returned by `list`, `inspect`, or `app active` when exact targeting matters. If a selector is omitted, most scoped actions operate on the active target. Prefer explicit selectors when more than one target could reasonably match the user's request.

Use `surface list` before a walkthrough or multi-step UI workflow. It reports both available and unavailable destinations with stable names and reasons. The direct `surface ... open` commands are idempotent; use them instead of toggle commands when the final open state matters. `surface list` accepts `--instance` or `--pid` for process selection but rejects window, tab, pane, and session selectors.

## Safety and limitations

- Invoke close actions only when the user explicitly asks to close something. Close actions flow through normal Brasa close behavior and may trigger existing app warnings.
- `input insert` and `input replace` only stage text. Brasa Control intentionally does not provide an action that submits or runs the input.
- Do not invent unsupported commands. Use the matching group's `help` first, then use `action list` or `action inspect` only when no dedicated group matches.
- Brasa Control affects only a running local Brasa application owned by the same user. It does not control remote or cloud Brasa instances.
- Each channel-specific Brasa Control CLI lists and targets only Brasa instances from its own channel.
- On Windows, local-control publication is disabled until authenticated broker transport is supported.

## Manual setup and troubleshooting

Brasa Control availability depends on the build channel and the **Settings > Scripting** toggle. The local-control mode defaults to enabled on internal dogfood builds (e.g., BrasaDev) and disabled on public channels (Stable, Preview, OSS). On any channel, the final gate is the **Settings > Scripting** toggle. The installed `{{brasactrl_binary_name}}` wrapper invokes the matching channel-specific Brasa executable.

If `{{brasactrl_binary_name}} instance list` is empty, confirm that a compatible same-channel Brasa app is running and Scripting is enabled. If a command reports multiple instances, rerun it with `--instance <instance_id>`.

If the symlink is not on `PATH`, follow the confirmation-gated setup flow in **How to invoke Brasa Control** or use `{{brasactrl_wrapper_path}}` directly.
