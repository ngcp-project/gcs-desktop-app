describe("Map annotations", () => {
  before(async () => {
    // Load Map Page
    await browser.url('/#/StaticScreen');
  });

  it("should create a new mission", async () => {
    await $('button.add-mission-button').click();
    browser.pause(1000); // wait a second to ensure mission list updates
    const missionsList = await $$('div.missions-list > div');
    const testMission = missionsList[missionsList.length - 1];
    await expect(await testMission.$('h3 > input').getValue()).toEqual('new mission');
  });

  it("should delete the mission it created", async () => {
    const missionsList = await $$('div.missions-list > div');
    const testMission = missionsList[missionsList.length - 1];
    await expect(await testMission.$('h3 > input').getValue()).toEqual('new mission');

    const oldLength = missionsList.length;
    await testMission.$('h3 > div > svg').click();
    browser.pause(1000); // wait a second to ensure mission list updates
    const newLength = (await $$('div.missions-list > div')).length;
    expect(newLength == oldLength - 1).toBe(true);
  });
});
