import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

import type { Action } from "../lib/data/reducer";
import { useReducerWithMiddleware } from "../lib/data/reducer";
import type { Page, ScreenData } from "../lib/data/ScreenData";
import styles from "../style/app.module.scss";
import { LTScreen } from "./ltscreen/LTScreen";
import { WaitingScreen } from "./waiting/WaitingScreen";

function selectPage(page: Page): React.FC<{ state: ScreenData }> {
  switch (page) {
    case "LTScreen":
      return LTScreen;
    case "WaitingScreen":
      return WaitingScreen;
  }
}

function App(): JSX.Element {
  const [state, dispatch] = useReducerWithMiddleware();

  const CurrentPage = selectPage(state.transition.current);
  const TransitingPage = state.transition.to && selectPage(state.transition.to);

  useEffect(() => {
    const unlisten = listen("event", (data) => {
      dispatch(JSON.parse(data.payload as string) as Action);
    });

    return () => {
      void unlisten.then((f) => f());
    };
  }, [dispatch]);

  return (
    <div className={styles.screen_container}>
      <CurrentPage state={state} />
      {TransitingPage && (
        <div className={styles.transiting}>
          <TransitingPage state={state} />
        </div>
      )}
    </div>
  );
}

export default App;
