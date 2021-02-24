import React from "react";
import { initialState, reducer } from "../lib/data/reducer";
import {LTScreen} from "./ltscreen/LTScreen";

import styles from "../style/app.module.scss";
import {WaitingScreen} from "./waiting/WaitingScreen";

function App() {
  const [state, dispatch] = React.useReducer(reducer, initialState);

  return (
    <div className={styles.screen_container}>
      <WaitingScreen state={state}/>
      {/*<LTScreen state={state} />*/}
    </div>
  );
}

export default App;
