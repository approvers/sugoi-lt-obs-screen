import * as React from "react";
import styles from "../../../style/comps/main.module.scss";
import {Service, TimelineCard} from "../../../lib/data/ScreenData";
import {useWindowDimensions} from "../../../lib/WindowHooks";

const icon: { [key in Service]: string } = {
  "twitter": "/font-awesome/twitter-brands.svg",
  "discord": "/font-awesome/discord-brands.svg",
  "youtube": "/font-awesome/youtube-brands.svg",
}

type TimelineProps = {
  timeline: TimelineCard[]
};
export const Timeline: React.FC<TimelineProps> = ({timeline}) => {
  const {width} = useWindowDimensions();

  // TODO: この数値をどうにかしたい
  const netWidth = width * 0.8075;

  return (
    <div className={styles.timeline} style={{height: `${netWidth * (9 / 16)}px`}}>
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
                <span className={styles[`card_service_${card.service}`]}>
                  <svg>
                    <use xlinkHref={icon[card.service] + "#icon"} />
                  </svg>
                </span>
              </div>
            </div>
          )
        )
      }
    </div>
  );
}
