import { createDriver, baseUrl } from '../helpers/driver.js';
import { expect } from 'chai';
import { By, until } from 'selenium-webdriver';

describe("Map annotations", () => {
  let driver;

  before(async () => {
    // Create the driver
    driver = await createDriver();
    // Load map page
    await driver.get(baseUrl + '/#/StaticScreen');
    // Set timeout to let elements load
    await driver.manage().setTimeouts({ implicit: 2000 });
  });

  after(async () => {
    // Stop the driver
    await driver.quit();
  });


  it("should create a new mission", async () => {
    await driver.findElement(By.css('button.add-mission-button')).click();
    await driver.sleep(500); // wait a tick to ensure mission list updates
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal('new mission');
  });

  it("should delete the mission it created", async () => {
    const missionsList = await driver.findElements(By.css('div.missions-list > div'));
    const testMission = missionsList[missionsList.length - 1];
    const missionName = await testMission.findElement(By.css('h3 > input')).getAttribute('value');
    expect(missionName).to.equal('new mission');

    const oldLength = missionsList.length;
    await testMission.findElement(By.css('h3 > div > svg')).click();
    await driver.sleep(500); // wait a tick to ensure mission list updates
    const newLength = (await driver.findElements(By.css('div.missions-list > div'))).length;
    expect(newLength).to.equal(oldLength - 1);
  });
});
