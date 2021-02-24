import * as React from "react";
import styles from "../../style/waiting/waiting_screen.module.scss";
import { Title } from "./parts/Title";
import {PresenterList} from "./parts/PresenterList";
import { ScreenData } from "../../lib/data/ScreenData";

type WaitingScreenProps = {
  state: ScreenData
};
export const WaitingScreen: React.FC<WaitingScreenProps> = ({state}) => (
  <div className={styles.wrapper}>
    <img className={styles.background} src={"/evil_spirits.png"} />
    <Title message={state.notification ?? "しばらくおまちください"}/>
    <PresenterList presentations={state.pending_presentation}/>
  </div>
)
