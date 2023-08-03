import * as React from "react";

import { Presentation } from "../../../lib/data/ScreenData";
import styles from "../../../style/waiting/waiting_screen.module.scss";
import { PresenterListElement } from "./PresenterListElement";

type PresenterListProps = {
  presentations: Array<Presentation>;
};
export const PresenterList: React.FC<PresenterListProps> = ({
  presentations,
}) => {
  if (presentations.length === 0) return null;

  return (
    <div className={styles.list}>
      <p className={styles.list_title}>今後の登壇予定</p>
      <div className={styles.list_wrapper}>
        {presentations.map((elem, index) => (
          <PresenterListElement key={index} presentation={elem} />
        ))}
      </div>
    </div>
  );
};
