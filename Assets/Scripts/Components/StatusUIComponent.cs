#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.WorldObjects.Core;
    using TMPro;
    using UnityEngine;
    using YamlDotNet.Serialization;
    using YamlDotNet.Serialization.NamingConventions;

    public class StatusUIComponent : MonoBehaviour
    {
        // FIELDS //

        private GameObject childObject;
        private GameObject userInterface;
        private RectTransform rectTransform;
        private TextMeshProUGUI textMeshPro;
        private ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(PascalCaseNamingConvention.Instance)
            .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
            .Build();

        // FUNCTIONS //

        public void Instantiate(GameObject userInterface)
        {
            this.userInterface = userInterface;

            this.childObject = new("StatusUIComponent");
            this.childObject.transform.SetParent(this.userInterface.transform);
            this.childObject.layer = 5; // UI layer

            this.rectTransform = this.childObject.AddComponent<RectTransform>();
            this.rectTransform.sizeDelta = new Vector2(320, 400);
            this.rectTransform.localScale = new Vector3(1, 1, 1);
            this.rectTransform.localPosition = new Vector3(330, -100, 10);
            this.rectTransform.rotation = Quaternion.Euler(0, 0, 0);
            this.rectTransform.anchorMin = new Vector2(0, 1);
            this.rectTransform.anchorMax = new Vector2(0, 1);
            this.rectTransform.pivot = new Vector2(1, 1);

            this.textMeshPro = this.childObject.AddComponent<TextMeshProUGUI>();
            this.textMeshPro.fontSize = 16;
            this.textMeshPro.color = new Color(1, 1, 1, 1);
            this.textMeshPro.overflowMode = TextOverflowModes.Truncate;
        }

        public void Display(List<WorldObjectCore> worldObjects)
        {
            // Nothing is here
            if (worldObjects == null)
            {
                this.textMeshPro.SetText("");
            }

            // Get the status data from each object
            List<StatusDataComponentCore.StatusData> statusDataList =
                worldObjects
                    ?.Where(worldObject => worldObject != null)
                    .Where(worldObject => worldObject.Status != null)
                    .Where(worldObject => worldObject.Status.Data != null)
                    .Select(worldObject => worldObject.Status.Data?.Invoke())
                    .Where(data => data != null)
                    .ToList() ?? new List<StatusDataComponentCore.StatusData>();

            if (statusDataList.Count == 0)
            {
                // If there are no statuses, clear the status
                this.textMeshPro.SetText("");
            }
            else
            {
                // Serialize the status list to YAML, then update the status
                string statusYaml = this.serializer.Serialize(statusDataList);
                this.textMeshPro.SetText(statusYaml);
            }
        }
    }
}
#endif
