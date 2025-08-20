import React from "react";
import EventForm from "./_8_EventForm";
import { useEvent } from "./useEvent";
import { useNavigate } from "@tanstack/react-router";

interface Props {
  eventId: string;
}

const EditEvent: React.FC<Props> = ({ eventId }) => {
  const { event, isLoading, error } = useEvent(eventId);
  const navigate = useNavigate();

  if (isLoading) return <p>Loading...</p>;
  if (error || !event) return <p>Event not found.</p>;

  return (
    <div className="p-4">
      <h2 className="text-xl font-bold mb-4">Edit Event</h2>
      <EventForm
        mode="edit"
        event={event}
        onSuccess={() => navigate({ to: "/admin/events/$eventId", params: { eventId } })}
      />
    </div>
  );
};

export default EditEvent;
