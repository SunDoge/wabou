import { batch, createSignal, type Accessor } from "solid-js";

export interface UpdateResult<State, Command = never> {
  state: State;
  commands: readonly Command[];
}

export type Update<State, Event, Command = never> = (
  state: State,
  event: Event,
) => UpdateResult<State, Command>;

export interface MachineOptions<State, Event, Command> {
  initialState: State;
  update: Update<State, Event, Command>;
  execute?: (command: Command, send: (event: Event) => void) => void;
  onTransition?: (result: UpdateResult<State, Command>, event: Event) => void;
}

export interface Machine<State, Event> {
  state: Accessor<State>;
  send(event: Event): boolean;
}

/** Solid adapter for an Elm-style pure update function and explicit commands. */
export function createMachine<State, Event, Command = never>(
  options: MachineOptions<State, Event, Command>,
): Machine<State, Event> {
  const [state, setState] = createSignal(options.initialState);
  const send = (event: Event) => {
    const previous = state();
    const result = options.update(previous, event);
    const changed = !Object.is(previous, result.state);
    batch(() => {
      if (changed) setState(() => result.state);
      options.onTransition?.(result, event);
    });
    for (const command of result.commands) options.execute?.(command, send);
    return changed;
  };
  return { state, send };
}

export function unchanged<State, Command = never>(
  state: State,
): UpdateResult<State, Command> {
  return { state, commands: [] };
}
