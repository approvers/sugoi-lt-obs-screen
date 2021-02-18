import * as React from "react";
import styles from "../../../style/comps/main.module.scss";
import {TimelineCard} from "../../../lib/data/ScreenData";

type TimelineProps = {
  timeline: TimelineCard[]
};
export const Timeline: React.FC<TimelineProps> = ({timeline}) => (
  <div className={styles.timeline}>
    {
      timeline.map(
        (card, index) => (
          <div className={styles.card} key={index}>
            <p className={styles.card_content}>{card.content}</p>
            <div className={styles.card_detail}>
              <span>
                {card.user.name}
                (<span className={styles.card_ident}>@{card.user.identifier}</span>)
              </span>
              <span>
                {card.service.charAt(0)}
              </span>
            </div>
          </div>
        )
      )
    }
  </div>
)
