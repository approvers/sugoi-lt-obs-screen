import {Presentation, ScreenData, TimelineCard} from "./ScreenData";

export type Action =
  {
    type: "notification.update",
    args: {
      new: string
    }
  } |
  {
    type: "presentation.update",
    args: {
      new: Presentation
    }
  } |
  {
    type: "timeline.add",
    args: {
      new: TimelineCard
    }
  } |
  {
    type: "timeline.flush",
    args: never
  };

export const initialState = ({
  presentation: {
    presenter: {
      name: "[Presenter Name]",
      identifier: "[Presenter Identifier]",
      userIcon: "https://pbs.twimg.com/profile_images/1328732285160472576/PiG9XbZ7_400x400.jpg"
    },
    title: "[Presentation Title]"
  },
  timeline: [],
  notification: "[Notification]"
});

export function reducer(state: ScreenData, action: Action): ScreenData {
  switch (action.type) {
    case "notification.update":
      return {
        ...state,
        notification: action.args.new,
      }
    case "timeline.add":
      return {
        ...state,
        timeline: [...state.timeline, action.args.new],
      }
    case "timeline.flush":
      return {
        ...state,
        timeline: []
      }
    case "presentation.update":
      return {
        ...state,
        presentation: action.args.new,
      }
  }
  return state;
}
