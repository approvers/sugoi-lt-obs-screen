import * as React from "react";

import { ScreenData } from "../../lib/data/ScreenData";
import { Footer } from "./parts/Footer";
import { Main } from "./parts/Main";
import { Notification } from "./parts/Notification";

type LTScreenProps = {
  state: ScreenData;
};

export const LTScreen: React.FC<LTScreenProps> = ({ state }) => (
  <div>
    <Notification notification={state.notification ?? ""} />
    <Main timeline={state.timeline} />
    <Footer presentation={state.presentation} />
  </div>
);
