import React, { useState } from "react";

type Event = {
  id: string;
  name: string;
  description?: string | null;
  location?: string | null;
  capacity: number;
  start_time: string;
  end_time: string;
};

interface Props {
  event?: Event;
  mode: "create" | "edit";
  onSuccess: (eventId: string) => void;
}

const EventForm: React.FC<Props> = ({ event, mode, onSuccess }) => {
  const [form, setForm] = useState({
    name: event?.name ?? "",
    description: event?.description ?? "",
    location: event?.location ?? "",
    capacity: event ? String(event.capacity) : "",
    start_time: event ? event.start_time.slice(0, 16) : "",
    end_time: event ? event.end_time.slice(0, 16) : "",
  });
  const [error, setError] = useState<string | null>(null);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    const payload = {
      name: form.name,
      description: form.description || null,
      location: form.location || null,
      capacity: Number(form.capacity),
      start_time: new Date(form.start_time).toISOString(),
      end_time: new Date(form.end_time).toISOString(),
    };
    const url =
      mode === "create"
        ? "http://localhost:8000/events"
        : `http://localhost:8000/events/${event?.id}`;
    const method = mode === "create" ? "POST" : "PUT";
    const res = await fetch(url, {
      method,
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (res.ok) {
      const data = await res.json();
      onSuccess(data.id);
    } else {
      setError("Failed to save event");
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 max-w-md">
      <div>
        <label className="block text-sm font-medium text-gray-700">Name</label>
        <input
          type="text"
          name="name"
          value={form.name}
          onChange={handleChange}
          className="mt-1 w-full border rounded p-2"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Description</label>
        <textarea
          name="description"
          value={form.description}
          onChange={handleChange}
          className="mt-1 w-full border rounded p-2"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Location</label>
        <input
          type="text"
          name="location"
          value={form.location}
          onChange={handleChange}
          className="mt-1 w-full border rounded p-2"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Capacity</label>
        <input
          type="number"
          name="capacity"
          value={form.capacity}
          onChange={handleChange}
          className="mt-1 w-full border rounded p-2"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">Start Time</label>
        <input
          type="datetime-local"
          name="start_time"
          value={form.start_time}
          onChange={handleChange}
          className="mt-1 w-full border rounded p-2"
        />
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700">End Time</label>
        <input
          type="datetime-local"
          name="end_time"
          value={form.end_time}
          onChange={handleChange}
          className="mt-1 w-full border rounded p-2"
        />
      </div>
      {error && <p className="text-red-500">{error}</p>}
      <button
        type="submit"
        className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
      >
        {mode === "create" ? "Create Event" : "Save Changes"}
      </button>
    </form>
  );
};

export default EventForm;
