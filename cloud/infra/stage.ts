export const domain = (() => {
  if ($app.stage === "production") return "nestri.io"
  if ($app.stage === "dev") return "dev.nestri.io"
  return `${$app.stage}.dev.nestri.io`
})()
