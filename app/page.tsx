
export default function Home() {
  return (
    <main>
      <h1>
        Welcome to the NEO explorer.
      </h1>
      <form action="/neo" method="get">
        <label htmlFor="neo_date">Date</label>
        <input type="date" name="neo_date" id="neo_date"></input>
        <p>
          <input type="submit" value="Submit"></input>
        </p>

      </form>
    </main>
  )
}
