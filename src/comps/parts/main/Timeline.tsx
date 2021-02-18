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
        (card, index) => <div key={index}>
          <p>{card.content}</p>
          <div>
            {card.user.name} (@{card.user.identifier}) has spoken via {card.service}
          </div>
        </div>
      )
    }
  </div>
)
