#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using Assets.Scripts.Core;
    using Assets.Scripts.Unity;
    using TMPro;
    using UnityEngine;
    using YamlDotNet.Serialization;
    using YamlDotNet.Serialization.NamingConventions;

    public class AlertComponent : MonoBehaviour
    {
        // STATICS //

        public static int maxAlerts = 10;

        // FIELDS //

        private GameObject childObject;
        private Canvas canvas;
        private RectTransform rectTransform;
        private TextMeshPro textMeshPro;
        private ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(PascalCaseNamingConvention.Instance)
            .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
            .Build();

        // PROPERTIES //

        public List<Dictionary<int, string>> Alerts;

        // FUNCTIONS //

        public void Instantiate()
        {
            this.childObject = new("AlertComponent");
            this.childObject.transform.SetParent(this.transform);
            this.childObject.transform.localPosition = Vector3.zero;
            this.childObject.transform.localScale = Vector3.one;
            this.childObject.transform.localRotation = Quaternion.identity;
            this.childObject.layer = 5; // UI layer

            this.canvas = this.childObject.AddComponent<Canvas>();
            this.canvas.renderMode = RenderMode.WorldSpace;
            this.canvas.planeDistance = 0.5f;
            this.canvas.sortingOrder = 0;

            // TODO: this rotates with the world object right now,
            // so when the world object rotates, the text rotates too.
            // which means that the text is often sideways or upside down.
            this.rectTransform = this.childObject.GetComponent<RectTransform>();
            this.rectTransform.sizeDelta = new Vector2(100, 100);
            this.rectTransform.localScale = new Vector3(0.01f, 0.01f, 0.01f);
            this.rectTransform.localPosition = new Vector3(0, 0, 0);
            this.rectTransform.rotation = Quaternion.Euler(0, 0, 0);

            this.textMeshPro = this.childObject.AddComponent<TextMeshPro>();
            this.textMeshPro.fontSize = 20;
            this.textMeshPro.color = new Color(1, 1, 1, 1);
        }

        public void Destroy()
        {
            Destroy(this.childObject);
            Destroy(this.canvas);
            Destroy(this.rectTransform);
            Destroy(this.textMeshPro);
        }

        public void Add(GameController gameController, string alert)
        {
            this.Alerts ??= new List<Dictionary<int, string>>();

            // Remove the oldest alert if we have too many
            if (this.Alerts.Count >= AlertComponent.maxAlerts)
            {
                this.Alerts.RemoveAt(0);
            }

            // If the incoming alert has the same text as the last alert,
            // then we don't need to add it again.
            if (this.Alerts.Count > 0 && this.Alerts[^1].ContainsValue(alert))
            {
                return;
            }

            Dictionary<int, string> alertDict = new() { { gameController.Tick, alert } };
            this.Alerts.Add(alertDict);

            // string alertsYaml = this.serializer.Serialize(this.Alerts);
            // this.textMeshPro.text = alertsYaml;
        }
    }
}
#endif
