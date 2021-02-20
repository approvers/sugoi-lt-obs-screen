import React from "react";
import { Notification } from "./parts/Notification";
import { Main } from "./parts/Main";
import { Footer } from "./parts/Footer";
import { Action, initialState, reducer } from "../lib/data/reducer";

function App() {
  const [state, dispatch] = React.useReducer(reducer, initialState);

  return (
    <>
      <Notification notification={state.notification ?? ""} />
      <Main timeline={state.timeline} />
      <Footer presentation={state.presentation} />
    </>
  );
}

export default App;
