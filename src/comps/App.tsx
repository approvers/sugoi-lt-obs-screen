import React from "react";
import {useReducerWithMiddleware} from "../lib/data/reducer";
import {LTScreen} from "./ltscreen/LTScreen";

import styles from "../style/app.module.scss";
import {WaitingScreen} from "./waiting/WaitingScreen";
import {Page, ScreenData} from "../lib/data/ScreenData";

function selectPage(page: Page): React.FC<{state: ScreenData}> {
  switch (page) {
    case "LTScreen":
      return LTScreen
    case "WaitingScreen":
      return WaitingScreen
  }
}

function App() {

  const [state, dispatch] = useReducerWithMiddleware();

  const CurrentPage = selectPage(state.transition.current);
  const TransitingPage = state.transition.to && selectPage(state.transition.to);

  return (
    <div className={styles.screen_container}>
      <CurrentPage state={state} />
      {TransitingPage && <div className={styles.transiting}>
        <TransitingPage state={state} />
      </div>}
    </div>
  );
}

export default App;
