describe("Example Map Test", () => {
  it("can find the missions header", async () => {
    await browser.url('/#/StaticScreen');
    const header = await $('div[data-sidebar="header"] div span');
    const text = await header.getText();
    expect(text).toMatch(/^Missions/);
  });
});
