import { Presentation, ScreenData, TimelineCard } from "./ScreenData";

export type Action =
  | {
      type: "notification.update";
      args: {
        new: string;
      };
    }
  | {
      type: "presentation.update";
      args: {
        new: Presentation;
      };
    }
  | {
      type: "timeline.add";
      args: {
        new: TimelineCard;
      };
    }
  | {
      type: "timeline.flush";
      args: never;
    };

export const initialState: ScreenData = {
  presentation: {
    presenter: {
      name: "フライさん",
      identifier: "loxygen_k",
      userIcon:
        "https://pbs.twimg.com/profile_images/1328732285160472576/PiG9XbZ7_400x400.jpg",
    },
    title: "いつの間にか議事録に「受胎宣告」と書いていた話",
    icon_fit_position: 0.6,
  },
  timeline: [
    {
      service: "youtube",
      user: {
        userIcon: undefined,
        identifier: "extremely_long_name_of_user_who_did_comment_to_this_event",
        name: "本イベントに関わる発言を行ったユーザが使用するユーザ名",
      },
      content: "Some random content here",
    },
    {
      service: "discord",
      user: {
        userIcon:
          "https://pbs.twimg.com/profile_images/1328732285160472576/PiG9XbZ7_400x400.jpg",
        identifier: "[ident2]",
        name: "Name2",
      },
      content: "Some random content here",
    },
    {
      service: "twitter",
      user: {
        userIcon:
          "https://pbs.twimg.com/profile_images/1328732285160472576/PiG9XbZ7_400x400.jpg",
        identifier: "[ident3]",
        name: "Name3",
      },
      content: "Some random content here",
    },
  ],
  notification: "[Notification]",
};

export function reducer(state: ScreenData, action: Action): ScreenData {
  switch (action.type) {
    case "notification.update":
      return {
        ...state,
        notification: action.args.new,
      };
    case "timeline.add":
      return {
        ...state,
        timeline: [...state.timeline, action.args.new],
      };
    case "timeline.flush":
      return {
        ...state,
        timeline: [],
      };
    case "presentation.update":
      return {
        ...state,
        presentation: action.args.new,
      };
  }
  return state;
}
