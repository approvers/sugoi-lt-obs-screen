import * as React from "react";
import styles from "../../../style/waiting/waiting_screen.module.scss";

export const Title = () => (
  <div className={styles.title}>
    <img className={styles.title_logo} src={"/approvers_icon.png"} />
    <p className={styles.title_subtitle}>限界開発鯖 1周年記念LT企画</p>
    <p className={styles.title_title}>第2回 限界LT</p>
    <p className={styles.title_message}>開始までしばらくおまちください</p>
  </div>
)
