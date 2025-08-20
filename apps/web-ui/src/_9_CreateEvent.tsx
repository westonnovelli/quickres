import React, { useState } from "react";
import EventForm from "./_8_EventForm";
import { Link } from "@tanstack/react-router";

const CreateEvent: React.FC = () => {
  const [eventId, setEventId] = useState<string | null>(null);

  if (eventId) {
    const shareLink = `${window.location.origin}/event/${eventId}`;
    return (
      <div className="p-4 space-y-4 text-center">
        <p className="font-semibold">Event created!</p>
        <p className="break-all">{shareLink}</p>
        <Link
          to="/admin/events/$eventId"
          params={{ eventId }}
          className="text-blue-600 underline"
        >
          Manage Event
        </Link>
      </div>
    );
  }

  return (
    <div className="p-4">
      <h2 className="text-xl font-bold mb-4">Create Event</h2>
      <EventForm mode="create" onSuccess={setEventId} />
    </div>
  );
};

export default CreateEvent;
