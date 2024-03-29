import { Dispatch, useReducer } from "react";

import { Page, Presentation, ScreenData, TimelineCard } from "./ScreenData";

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
    }
  | {
      type: "waiting.message.update";
      args: {
        new: string;
      };
    }
  | {
      type: "waiting.pending.update";
      args: {
        new: Array<Presentation>;
      };
    }
  | {
      type: "screen.update";
      args: {
        new: Page;
      };
    }
  | {
      type: "screen.startTransition"; // Internal use
      args: {
        new: Page;
      };
    }
  | {
      type: "screen.finishTransition"; // Internal use
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
  pending_presentation: [
    {
      presenter: {
        name: "フライさん",
        identifier: "loxygen_k",
      },
      title: "いつの間にか議事録に「受胎宣告」と書いていた話",
    },
    {
      presenter: {
        name: "Hoge F. Piyo",
        identifier: "hogepiyo",
      },
      title: "突然現れ突然消えた「受胎宣告」― 一瞬の間に何があったのか",
    },
    {
      presenter: {
        name: "Foo B. Corge",
        identifier: "foo_corge",
      },
      title: "「受胎宣告」事件について考える",
    },
  ],
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
  notification: "開始までしばらくおまちください",
  transition: {
    current: "WaitingScreen",
  },
};

function middleware(
  state: ScreenData,
  action: Action,
  dispatch: Dispatch<Action>,
): ScreenData {
  switch (action.type) {
    case "screen.update":
      setTimeout(
        () =>
          dispatch({
            type: "screen.finishTransition",
          }),
        2000,
      );
      dispatch({
        type: "screen.startTransition",
        args: {
          new: action.args.new,
        },
      });
      break;
    default:
      // Do nothing
      break;
  }
  return state;
}

function reducer(state: ScreenData, action: Action): ScreenData {
  switch (action.type) {
    case "notification.update":
    case "waiting.message.update":
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
    case "waiting.pending.update":
      return {
        ...state,
        pending_presentation: action.args.new,
      };
    case "screen.update":
      return state;
    case "screen.startTransition":
      return {
        ...state,
        transition: {
          ...state.transition,
          to: action.args.new,
        },
      };
    case "screen.finishTransition":
      return {
        ...state,
        transition: {
          current: state.transition.to ?? state.transition.current,
          to: undefined,
        },
      };
  }
}

export function useReducerWithMiddleware(): [
  ScreenData,
  (action: Action) => void,
] {
  const [state, dispatch] = useReducer(reducer, initialState);

  const dispatchWithMiddleware = (action: Action) => {
    middleware(state, action, dispatch);
    dispatch(action);
  };

  return [state, dispatchWithMiddleware];
}
