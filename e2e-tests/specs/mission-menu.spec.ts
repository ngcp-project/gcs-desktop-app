import { createDriver, baseUrl } from '../helpers/driver.js';
import { expect } from 'chai';
import { By, Origin, until } from 'selenium-webdriver';

describe("Mission menu", () => {
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

  const pauseDuration = 50;
  const testMissionName = "foo test mission";
  const testStageName = "bar test stage";
  const stageAmount = 5;

  it("should create a new mission", async () => {
    await driver.findElement(By.css('button.add-mission-button')).click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal('new mission');
  });

  it("should rename the mission", async () => {
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionNameEl = await testMission.findElement(By.css('h3 > input'));
    const missionName = await missionNameEl.getAttribute('value');
    expect(missionName).to.equal('new mission');

    await missionNameEl.clear();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates

    const tempMissionsList = await driver.findElements(By.css('div.missions-list > div'));
    const tempMission = tempMissionsList[tempMissionsList.length - 1];
    await tempMission.findElement(By.css('h3 > input')).sendKeys(testMissionName);
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates

    const newMissionsList = await driver.findElements(By.css('div.missions-list > div'));
    const newTestMission = newMissionsList[newMissionsList.length - 1];
    const newName = await newTestMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(newName).to.equal(testMissionName);
  });


  it("should load the vehicles list", async () => {
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal(testMissionName);

    await testMission.click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    await driver.findElements(By.css('div.vehicles-list > div'));
  });

  const testCases = [
    { name: "ERU", index: 0 },
    { name: "MEA", index: 1 },
    { name: "MRA", index: 2 }
  ];

  testCases.forEach(({ name, index }) => {

    it(`should see the ${name} in the vehicles list`, async () => {
      const vehicleCard = (await driver.findElements(By.css('div.vehicles-list > div')))[index];
      const vehicleCardName = await vehicleCard.findElement(By.css('h3')).getText();
      expect(vehicleCardName).to.equal(name);
    });

    it(`should load the stages list for the ${name}`, async () => {
      const vehicleCard = (await driver.findElements(By.css('div.vehicles-list > div')))[index];
      const vehicleCardName = await vehicleCard.findElement(By.css('h3')).getText();
      expect(vehicleCardName).to.equal(name);

      await vehicleCard.click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      await driver.findElement(By.css('button.add-stage-button')); // check for the button instead because the list doesn't exist yet
    });

    it(`should create a new stage for the ${name}`, async () => {
      await driver.findElement(By.css('button.add-stage-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      const stagesList = await driver.findElements(By.css('div.stages-list > div'));
      const testStage = stagesList[stagesList.length - 1];
      const stageName = await testStage.findElement(By.css('h3 > input')).getAttribute('value');
      expect(stageName).to.equal('New Stage');
    });

    it(`should rename the stage for the ${name}`, async () => {
      const stagesList = await driver.findElements(By.css('div.stages-list > div'));
      const testStage = stagesList[stagesList.length - 1];
      const stageNameEl = await testStage.findElement(By.css('h3 > input'));
      const stageName = await stageNameEl.getAttribute('value');
      expect(stageName).to.equal('New Stage');

      await stageNameEl.clear();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates

      const tempStagesList = await driver.findElements(By.css('div.stages-list > div'));
      const tempStage = tempStagesList[tempStagesList.length - 1];
      await tempStage.findElement(By.css('h3 > input')).sendKeys(testStageName);
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates

      const newStagesList = await driver.findElements(By.css('div.stages-list > div'));
      const newTestStage = newStagesList[newStagesList.length - 1];
      const newName = await newTestStage.findElement(By.css('h3 > input')).getAttribute('value');
      expect(newName).to.equal(testStageName);
    });

    it(`should create the search area for the ${name}`, async () => {
      const stagesList = await driver.findElements(By.css('div.stages-list > div'));
      const testStage = stagesList[stagesList.length - 1];
      const stageName = await testStage.findElement(By.css('h3 > input')).getAttribute('value');
      expect(stageName).to.equal(testStageName);

      const drawOrigin = 200;
      const drawOffset = 100;
      const drawDuration = pauseDuration;

      await testStage.findElement(By.css('svg.setup-searcharea-button')).click();

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

    it(`should render and hide the search area for the ${name}`, async () => {
      // ASSUMPTION: only one search area is currently being rendered
      const stageMapPath = await driver.findElement(By.css('path.leaflet-interactive'));

      const stagesList = await driver.findElements(By.css('div.stages-list > div'));
      const testStage = stagesList[stagesList.length - 1];
      const stageName = await testStage.findElement(By.css('h3 > input')).getAttribute('value');
      expect(stageName).to.equal(testStageName);

      // hidden
      await testStage.findElement(By.css('svg.toggle-searcharea-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      expect(await stageMapPath.getAttribute('stroke-opacity')).to.equal('0');
      expect(await stageMapPath.getAttribute('fill-opacity')).to.equal('0');

      // shown
      await testStage.findElement(By.css('svg.toggle-searcharea-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      expect(await stageMapPath.getAttribute('stroke-opacity')).to.equal('1');
      expect(await stageMapPath.getAttribute('fill-opacity')).to.equal('0.2');
    });

    it(`should edit the search area for the ${name}`, async () => {
      const stagesList = await driver.findElements(By.css('div.stages-list > div'));
      const testStage = stagesList[stagesList.length - 1];
      const stageName = await testStage.findElement(By.css('h3 > input')).getAttribute('value');
      expect(stageName).to.equal(testStageName);

      const vertexIndex = 6;
      const drawOffset = 50;
      const drawDuration = pauseDuration;

      await testStage.findElement(By.css('svg.setup-searcharea-button')).click();

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

      await testStage.findElement(By.css('svg.setup-searcharea-button')).click();
      await driver.wait(until.stalenessOf(vertexList[0]), pauseDuration); // all items in vertexList should no longer exist at this point
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    });

    it(`should delete the created stage for the ${name}`, async () => {
      const stagesList = await driver.findElements(By.css('div.stages-list > div'));
      const testStage = stagesList[stagesList.length - 1];
      const stageName = await testStage.findElement(By.css('h3 > input')).getAttribute('value');
      expect(stageName).to.equal(testStageName);

      const oldLength = stagesList.length;
      await testStage.findElement(By.css('svg.delete-stage-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      const newLength = (await driver.findElements(By.css('div.stages-list > div'))).length;
      expect(newLength).to.equal(oldLength - 1);
    });

    for (let i = 0; i < stageAmount; i++) {
      it(`should create and rename stage ${i} for the ${name}`, async () => {
        await driver.findElement(By.css('button.add-stage-button')).click();
        await driver.sleep(pauseDuration); // wait a tick to ensure list updates
        const stagesList = await driver.findElements(By.css('div.stages-list > div'));
        const testStage = stagesList[stagesList.length - 1];
        const stageNameEl = await testStage.findElement(By.css('h3 > input'));
        const stageName = await stageNameEl.getAttribute('value');
        expect(stageName).to.equal('New Stage');

        // webdriver doesn't let you interact with an element if it's offscreen
        // furthermore selenium's scroll action only works for the entire page, not specifc elements
        // therefore execute a javascript snippet that scrolls to the new stage
        driver.executeScript("arguments[0].scrollIntoView(true);", testStage);

        await stageNameEl.clear();
        await driver.sleep(pauseDuration); // wait a tick to ensure list updates

        const tempStagesList = await driver.findElements(By.css('div.stages-list > div'));
        const tempStage = tempStagesList[tempStagesList.length - 1];
        await tempStage.findElement(By.css('h3 > input')).sendKeys(`${testStageName} ${i}`);
        await driver.sleep(pauseDuration); // wait a tick to ensure list updates

        const newStagesList = await driver.findElements(By.css('div.stages-list > div'));
        const newTestStage = newStagesList[newStagesList.length - 1];
        const newName = await newTestStage.findElement(By.css('h3 > input')).getAttribute('value');
        expect(newName).to.equal(`${testStageName} ${i}`);
      });
    }

    for (let i = 1; i < stageAmount; i++) {
      it(`should advance from stage ${i - 1} to ${i} for the ${name}`, async () => {
        await driver.findElement(By.css('button.back-button')).click();
        await driver.sleep(pauseDuration); // wait a tick to ensure list updates
        const vehicleCard = (await driver.findElements(By.css('div.vehicles-list > div')))[index];
        const vehicleCardName = await vehicleCard.findElement(By.css('h3')).getText();
        expect(vehicleCardName).to.equal(name);

        const vehicleStage = await vehicleCard.findElement(By.css('span.current-stage')).getText();
        expect(vehicleStage).to.equal(`Stage: ${testStageName} ${i - 1}`);

        const vehicleNextStageButton = vehicleCard.findElement(By.css('button.next-stage-button'));

        // webdriver doesn't let you interact with an element if it's offscreen
        // furthermore selenium's scroll action only works for the entire page, not specifc elements
        // therefore execute a javascript snippet that scrolls to the new stage
        driver.executeScript("arguments[0].scrollIntoView(true);", vehicleNextStageButton);

        await vehicleNextStageButton.click();
        await driver.sleep(pauseDuration); // wait a tick to ensure list updates
        
        const newVehicleCard = (await driver.findElements(By.css('div.vehicles-list > div')))[index];
        const newVehicleStage = await newVehicleCard.findElement(By.css('span.current-stage')).getText();
        expect(newVehicleStage).to.equal(`Stage: ${testStageName} ${i}`);

        await newVehicleCard.click();
        await driver.sleep(pauseDuration); // wait a tick to ensure list updates
        await driver.findElement(By.css('button.add-stage-button')); // check for the button instead because the list doesn't exist yet
      });

      it(`should confirm stage statuses on stage ${i} for the ${name}`, async () => {
        var stagesList = await driver.findElements(By.css('div.stages-list > div'));
        await stagesList.forEach(async (stage, j) => {
          var expectedStatus = "";
          if (j > i) {
            expectedStatus = "Inactive";
          } else if (j < i) {
            expectedStatus = "Complete";
          } else {
            expectedStatus = "Active";
          }
          const stageStatus = await stage.findElement(By.css('span.stage-status')).getText();
          expect(stageStatus).to.equal(`Status: ${expectedStatus}`);
        });
      });
    }

    it("should return to vehicles list", async () => {
      await driver.findElement(By.css('button.back-button')).click();
      await driver.sleep(pauseDuration); // wait a tick to ensure list updates
      await driver.findElements(By.css('div.vehicles-list > div'));
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
    expect(missionName).to.equal(testMissionName);

    const oldLength = missionsList.length;
    await testMission.findElement(By.css('svg.delete-mission-button')).click();
    await driver.sleep(pauseDuration); // wait a tick to ensure list updates
    const newLength = (await driver.findElements(By.css('div.missions-list > div'))).length;
    expect(newLength).to.equal(oldLength - 1);
  });
});
