import React, { useState } from "react";
import { useEvent } from "./useEvent";
import { Link } from "@tanstack/react-router";

interface Props {
  eventId: string;
}

const ManageEvent: React.FC<Props> = ({ eventId }) => {
  const { event, isLoading, error } = useEvent(eventId);
  const [email, setEmail] = useState("");
  const [token, setToken] = useState<string | null>(null);

  if (isLoading) return <p>Loading...</p>;
  if (error || !event) return <p>Event not found.</p>;

  const shareLink = `${window.location.origin}/event/${eventId}`;

  const sendInvite = async (e: React.FormEvent) => {
    e.preventDefault();
    const res = await fetch(`http://localhost:8000/events/${eventId}/scanner_invites`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email }),
    });
    if (res.ok) {
      const data = await res.json();
      setToken(data.token);
      setEmail("");
    }
  };

  return (
    <div className="p-4 space-y-4">
      <h2 className="text-xl font-bold">{event.name}</h2>
      <p className="break-all">Share link: {shareLink}</p>
      <Link
        to="/admin/events/$eventId/edit"
        params={{ eventId }}
        className="text-blue-600 underline"
      >
        Edit Event
      </Link>
      <form onSubmit={sendInvite} className="space-y-2 max-w-md">
        <label className="block text-sm font-medium text-gray-700">
          Scanner Email
        </label>
        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          className="mt-1 w-full border rounded p-2"
        />
        <button
          type="submit"
          className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
        >
          Send Invite
        </button>
      </form>
      {token && <p className="text-sm break-all">Invite token: {token}</p>}
    </div>
  );
};

export default ManageEvent;
