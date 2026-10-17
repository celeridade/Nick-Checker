import { NextResponse } from "next/server";

export async function POST(req: Request) {
  let field = "email";
  try {
    const body = await req.json();
    if (typeof body?.field === "string" && body.field) field = body.field;
  } catch {
    /* default field */
  }
  return NextResponse.json({
    field,
    found: 3,
    results: [{ email: "dryann120@gmail.com", source: { name: "BreachForums" } }],
  });
}
