<html lang="en">
    <head>
        <meta charset="utf8"/>
    </head>
    <body>
        <h1>Users will see the following survey questions:</h1>
        <form method="post" action="/makeSurvey">
          {{#questions}}
            {{text}}<br><input type="text" name="q{{number}}"></br>
          {{/questions}}
          <button type="submit">Looks correct?</button>
          <button type="submit">Something wrong?</button><br>
        </form>
    </body>
</html>
