import * as React from "react";

import styles from "../../../style/waiting/waiting_screen.module.scss";

type TitleProps = {
  message: string;
};
export const Title: React.FC<TitleProps> = ({ message }) => (
  <div className={styles.title + " " + styles.title_border}>
    <img className={styles.title_logo} src={"/approvers_icon.png"} alt="" />
    <p className={styles.title_caption}>限界開発鯖 1周年記念LT企画</p>
    <p className={styles.title_title}>第2回 限界LT</p>
    <p className={styles.title_subtitle}>First Anniversary</p>
    <p className={styles.title_message}>{message}</p>
  </div>
);
