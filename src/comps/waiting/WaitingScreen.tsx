import * as React from "react";
import styles from "../../style/waiting/waiting_screen.module.scss";
import { Title } from "./parts/Title";

export const WaitingScreen = () => (
  <div className={styles.wrapper}>
    <img className={styles.background} src={"/evil_spirits.png"} />
    <Title />
  </div>
)
