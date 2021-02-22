import * as React from "react";
import { ScreenData } from "../../lib/data/ScreenData";
import { Notification } from "./parts/Notification";
import { Main } from "./parts/Main";
import { Footer } from "./parts/Footer";

type LTScreenProps = {
  state: ScreenData
};

export const LTScreen: React.FC<LTScreenProps> = ({state}) => (
  <>
    <Notification notification={state.notification ?? ""} />
    <Main timeline={state.timeline} />
    <Footer presentation={state.presentation} />
  </>
);
