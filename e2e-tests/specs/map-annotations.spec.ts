import { createDriver, baseUrl } from '../helpers/driver.js';
import { expect } from 'chai';
import { By, Origin, until } from 'selenium-webdriver';

describe("Map annotations", () => {
  let driver;

  before(async () => {
    // Set up the driver
    driver = await createDriver();
    await driver.manage().setTimeouts({ implicit: 2000 });
    // Load map page
    await driver.get(baseUrl + '/#/StaticScreen');
  });

  after(async () => {
    // Stop the driver
    await driver.quit();
  });

  const pauseDuration = 25;

  it("should create a new mission", async () => {
    await driver.findElement(By.css('button.add-mission-button')).click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal('new mission');
  });

  it("should load the zones list", async () => {
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal('new mission');

    await testMission.findElement(By.css('button.zones-mission-button')).click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    await driver.findElements(By.css('div.zones-list > div'));
  });

  const testCases = [
    { name: "Keep In", index: 0 },
    { name: "Keep Out", index: 1 }
  ];

  testCases.forEach(({ name, index }) => {

    it(`should see ${name} zones`, async () => {
      const zoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const zoneCardName = await zoneCard.findElement(By.css('h3')).getText();
      expect(zoneCardName).to.equal(name);
    });

    it(`should create a new ${name} zone`, async () => {
      const oldZoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      await oldZoneCard.findElement(By.css('button.add-zone-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates

      const zoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const zoneList = await zoneCard.findElements(By.css('div.zone-content > div'));
      const zone = zoneList[0];
      const zoneName = await zone.findElement(By.css('span')).getText();
      expect(zoneName).to.equal("Zone 0");
    });

    it(`should set up the created ${name} zone`, async () => {
      const drawOrigin = 200;
      const drawOffset = 100;
      const drawDuration = pauseDuration;

      const zoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const zone = (await zoneCard.findElements(By.css('div.zone-content > div')))[0];
      await zone.findElement(By.css('svg.setup-zone-button')).click();

      const actions = driver.actions({ async: true });
      await actions
        .move({ x: drawOrigin, y: drawOrigin, origin: Origin.VIEWPORT }).click().pause(drawDuration)
        .move({ x: drawOffset, y: 0, origin: Origin.POINTER }).click().pause(drawDuration)
        .move({ x: 0, y: drawOffset, origin: Origin.POINTER }).click().pause(drawDuration)
        .move({ x: 0 - drawOffset, y: 0, origin: Origin.POINTER }).click().pause(drawDuration)
        .move({ x: 0, y: 0 - drawOffset, origin: Origin.POINTER }).click().pause(drawDuration)
        .perform();
      await actions.clear();

      await driver.findElement(By.css('path.leaflet-interactive'));
    });

    it(`should render and hide the created ${name} zone`, async () => {
      // ASSUMPTION: only one zone is currently being rendered
      const zoneMapPath = await driver.findElement(By.css('path.leaflet-interactive'));

      const zoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const zone = (await zoneCard.findElements(By.css('div.zone-content > div')))[0];

      // hidden
      await zone.findElement(By.css('svg.toggle-zone-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      expect(await zoneMapPath.getAttribute('stroke-opacity')).to.equal('0');
      expect(await zoneMapPath.getAttribute('fill-opacity')).to.equal('0');

      // shown
      await zone.findElement(By.css('svg.toggle-zone-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      expect(await zoneMapPath.getAttribute('stroke-opacity')).to.equal('1');
      expect(await zoneMapPath.getAttribute('fill-opacity')).to.equal('0.2');
    });

    it(`should edit the created ${name} zone`, async () => {
      const vertexIndex = 6;
      const drawOffset = 50;
      const drawDuration = pauseDuration;

      const zoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const zone = (await zoneCard.findElements(By.css('div.zone-content > div')))[0];
      await zone.findElement(By.css('svg.setup-zone-button')).click();

      const vertexList = await driver.findElements(By.css('div.leaflet-marker-pane > div.leaflet-marker-draggable'));

      const vertex = vertexList[vertexIndex];
      const oldTransform = await vertex.getCssValue('transform');

      const actions = driver.actions({ async: true });
      await actions
        .move({ origin: vertex })
        .dragAndDrop(vertex, { x: drawOffset, y: drawOffset })
        .pause(drawDuration)
        .perform();
      await actions.clear();
      const newTransform = await vertex.getCssValue('transform');
      expect(newTransform).to.not.equal(oldTransform);

      await actions
        .contextClick(vertex)
        .pause(drawDuration)
        .perform();
      await actions.clear();
      // the element reference still exists even if the element gets deleted,
      // but "waiting" for staleness does check if it's removed
      await driver.wait(until.stalenessOf(vertex), pauseDuration);
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates

      await zone.findElement(By.css('svg.setup-zone-button')).click();
      await driver.wait(until.stalenessOf(vertexList[0]), pauseDuration); // all items in vertexList should no longer exist at this point
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    });

    it(`should delete the created ${name} zone`, async () => {
      const zoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const zoneList = await zoneCard.findElements(By.css('div.zone-content > div'));
      const zone = zoneList[0];
      const zoneName = await zone.findElement(By.css('span')).getText();
      expect(zoneName).to.equal("Zone 0");

      const oldLength = zoneList.length;
      await zone.findElement(By.css('svg.delete-zone-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      const newZoneCard = (await driver.findElements(By.css('div.zones-list > div')))[index];
      const newLength = (await newZoneCard.findElements(By.css('div.zone-content > div'))).length;
      expect(newLength).to.equal(oldLength - 1);
    });

  });

  it("should return to missions list", async () => {
    await driver.findElement(By.css('button.back-button')).click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    await driver.findElements(By.css('div.missions-list > div'));
  });

  it("should delete the created mission", async () => {
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal('new mission');

    const oldLength = missionsList.length;
    await testMission.findElement(By.css('svg.delete-mission-button')).click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    const newLength = (await driver.findElements(By.css('div.missions-list > div'))).length;
    expect(newLength).to.equal(oldLength - 1);
  });
});
