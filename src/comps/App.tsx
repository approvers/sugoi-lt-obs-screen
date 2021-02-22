import React from "react";
import { initialState, reducer } from "../lib/data/reducer";
import {LTScreen} from "./ltscreen/LTScreen";

function App() {
  const [state, dispatch] = React.useReducer(reducer, initialState);

  return (
    <LTScreen state={state} />
  );
}

export default App;
